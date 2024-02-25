use std::{
  env,
  fs,
  fs::File,
  io::{Read, Write},
  path::{Path, PathBuf},
};

use byteorder::{ByteOrder, NativeEndian};
use strum::{Display, EnumIter};
use tracing::*;

use super::stage::ShaderStage;
use crate::{
  vulkan::{error::VulkanError, shader::Shader},
  vulkan_error,
  vulkan_shader_error,
};

#[derive(EnumIter, Display, Clone, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum Source {
  // GLSL { path: PathBuf, source: String },
  SPIRV { path: PathBuf, words: Vec<u32> },
}

impl Source {
  pub fn new<S: ShaderStage>(path: impl Into<PathBuf>) -> Self {
    let path = path.into();
    match Self::read_from_cache::<S>(&path) {
      Ok(words) => Self::SPIRV { path, words },
      Err(_) => match Self::read_from_file::<S>(path.clone()) {
        Ok(words) => Self::SPIRV { path, words },
        Err(error) => {
          error!("{error} | Fallback shader being used");
          Self::read_default::<S>()
        }
      },
    }
  }

  pub fn from_source<S: ShaderStage>(path: impl Into<PathBuf>, source: &str) -> Self {
    let path = path.into();
    let mut new_path: String = path.to_str().unwrap_or_default().into();
    while new_path.starts_with("../") {
      new_path = new_path.trim_start_matches("../").to_string();
    }
    let Ok(cached_path) = Self::cached_path(new_path) else {
      return Self::read_default::<S>();
    };
    match Self::compile_shader_type::<S, _>(source, &cached_path) {
      Ok(bytes) => {
        let words = Self::to_words(bytes);
        Self::SPIRV { path, words }
      }
      Err(_) => Self::read_default::<S>(),
    }
  }
}

impl Source {
  pub fn read_default<S: ShaderStage>() -> Self {
    let path = S::default_path();
    match Self::read_from_cache::<S>(path.clone()) {
      Ok(words) => Self::SPIRV { path, words },
      Err(_) => {
        let cached_path = Self::cached_path(path.clone()).unwrap();
        let source = S::default_source();
        let bytes = Self::compile_shader_type::<S, _>(&source, &cached_path).unwrap();
        let words = Self::to_words(bytes);
        Self::SPIRV { path, words }
      }
    }
  }

  fn read_from_file<S: ShaderStage>(path: impl AsRef<Path>) -> Result<Vec<u32>, VulkanError> {
    let path = path.as_ref();
    let cached_path = Self::cached_path(path)?;
    let uncached_path = Self::relative_to_exe(path)?;

    trace!("[{:?}] Reading stage... {:?}", S::kind(), uncached_path);
    match File::open(uncached_path) {
      Ok(mut file) => {
        let mut source = String::new();
        file.read_to_string(&mut source)?;
        let bytes = Self::compile_shader_type::<S, _>(&source, &cached_path)?;
        match bytemuck::try_cast_slice(&bytes) {
          Ok(value) => Ok(value.to_vec()),
          Err(_) => Err(VulkanError::Shader("failed to cast bytes".into()))?,
        }
      }
      Err(error) => Err(VulkanError::from(error)),
    }
  }

  fn to_words(vec_8: Vec<u8>) -> Vec<u32> {
    let mut vec_32: Vec<u32> = vec![0; vec_8.len() / 4];
    NativeEndian::read_u32_into(&vec_8, &mut vec_32);
    vec_32
  }

  fn read_from_cache<S: ShaderStage>(path: impl AsRef<Path>) -> Result<Vec<u32>, VulkanError> {
    let path = path.as_ref();
    let cached_path = Self::cached_path(path)?;
    let uncached_path = Self::relative_to_exe(path)?;

    if cached_path.exists() && Self::cached_file_younger_than_exe(&cached_path)? {
      trace!("[{:?}] Reading cached stage... {:?}", S::kind(), uncached_path);
      match File::open(&cached_path) {
        Ok(mut file) => ash::util::read_spv(&mut file).map_err(VulkanError::from),
        Err(err) => Err(VulkanError::from(err)),
      }
    } else {
      Err(vulkan_shader_error!("[{:?}] failed to find cached stage.", S::kind()))
    }
  }

  fn relative_to_exe(path: impl Into<PathBuf>) -> Result<PathBuf, VulkanError> {
    let shader_dir = env::current_exe()?.parent().unwrap().join(path.into());

    Ok(shader_dir)
  }

  fn cached_path(path: impl AsRef<Path>) -> Result<PathBuf, VulkanError> {
    let shader_cache_dir = env::current_exe()?
      .parent()
      .ok_or_else(|| vulkan_error!("invalid exe parent"))?
      .join(Shader::SHADER_CACHE_DIR);

    let mut cached_path = shader_cache_dir.join(
      path
        .as_ref()
        .strip_prefix(Shader::SHADER_ASSET_DIR)
        .unwrap_or_else(|_error| path.as_ref()),
    );

    // let file_name = cached_path.file_name().context("invalid file name")?;
    let parent = cached_path
      .parent()
      .ok_or_else(|| vulkan_error!("invalid file parent"))?;

    info!("{:?}", cached_path);
    if let Err(error) = fs::create_dir_all(parent) {
      error!("{error}");
    };

    let mut extension = cached_path
      .extension()
      .ok_or_else(|| vulkan_error!("invalid file extension"))?
      .to_os_string();
    extension.push(".spv");

    cached_path.set_extension(extension);

    Ok(cached_path)
  }

  fn cached_file_younger_than_exe(cached_file: impl AsRef<Path>) -> Result<bool, VulkanError> {
    let file_age = cached_file.as_ref().metadata()?.modified()?;
    let exe_age = env::current_exe()?.metadata()?.modified()?;
    // debug!("File age: [{file_age:?}], Exe age: [{exe_age:?}]");
    Ok(file_age >= exe_age)
  }

  fn compile_shader_type<S: ShaderStage, P: AsRef<Path>>(
    source: &str,
    output_path: &P,
  ) -> Result<Vec<u8>, VulkanError> {
    let output_path = output_path.as_ref();
    let compiler = shaderc::Compiler::new().ok_or_else(|| vulkan_error!("failed to initialize shaderc compiler"))?;
    let mut options =
      shaderc::CompileOptions::new().ok_or_else(|| vulkan_error!("failed to initialize shaderc compiler options"))?;

    options.set_source_language(shaderc::SourceLanguage::GLSL);

    if !cfg!(debug_assertions) {
      options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    }

    match compiler.compile_into_spirv(
      source,
      S::kind().into(),
      output_path
        .file_name()
        .ok_or_else(|| vulkan_error!("failed to access file_name"))?
        .to_str()
        .ok_or_else(|| vulkan_error!("failed to convert file_name to str"))?,
      S::kind().entry_point().to_str().unwrap(),
      Some(&options),
    ) {
      Ok(result) => {
        trace!("[{:?}] Compiled stage.", S::kind());

        match std::fs::File::create(output_path) {
          Ok(mut file) => match file.write_all(result.as_binary_u8()) {
            Ok(_) => {
              trace!("[{:?}] Cached stage. {:?}", S::kind(), output_path)
            }
            Err(_) => error!("[{:?}] Failed to write stage to shader cache. {:?}", S::kind(), output_path),
          },
          Err(_) => error!("[{:?}] Failed to create file in shader cache. {:?}", S::kind(), output_path),
        }

        Ok(result.as_binary_u8().into())
      }
      Err(err) => {
        error!("[{:?}] Failed to compile stage: `{err:?}`", S::kind());
        Err(VulkanError::Shaderc(err))
      }
    }
  }
}

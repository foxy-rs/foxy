use std::{
  env, fs,
  fs::File,
  io::{Read, Write},
  path::{Path, PathBuf},
};

use anyhow::Context;
use byteorder::{ByteOrder, NativeEndian};
use strum::{Display, EnumIter};
use tracing::*;

use crate::{error::VulkanError, shader_error};

use super::{stage::StageInfo, storage::ShaderStore};

#[derive(EnumIter, Display, Clone, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum Source {
  // GLSL { path: PathBuf, source: String },
  SPIRV { path: PathBuf, words: Vec<u32> },
}

impl Source {
  pub fn new<S: StageInfo, P: Into<PathBuf>>(path: P) -> Self {
    let path = path.into();
    match Self::read_from_cache::<S, _>(&path) {
      Ok(words) => Self::SPIRV { path, words },
      Err(_) => match Self::read_from_file::<S, _>(path.clone()) {
        Ok(words) => Self::SPIRV { path, words },
        Err(error) => {
          error!("{error} | Fallback shader being used");
          Self::read_default::<S>()
        }
      },
    }
  }
}

impl Source {
  pub fn read_default<S: StageInfo>() -> Self {
    let path: PathBuf = format!("default_{}_shader", S::kind()).into();
    match Self::read_from_cache::<S, _>(path.clone()) {
      Ok(words) => Self::SPIRV { path, words },
      Err(_) => {
        let cached_path = Self::cached_path(path.clone()).unwrap();
        let path: PathBuf = format!("default_{}_shader", S::kind()).into();
        let source = S::default_source();
        let bytes = Self::compile_shader_type::<S, _>(&source, &cached_path).unwrap();
        let words = Self::to_words(bytes);
        Self::SPIRV { path, words }
      }
    }
  }

  fn read_from_file<S: StageInfo, P: AsRef<Path>>(path: P) -> Result<Vec<u32>, VulkanError> {
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

  fn read_from_cache<S: StageInfo, P: AsRef<Path>>(path: P) -> Result<Vec<u32>, VulkanError> {
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
      Err(shader_error!("[{:?}] failed to find cached stage.", S::kind()))
    }
  }

  fn relative_to_exe<P: Into<PathBuf>>(path: P) -> Result<PathBuf, VulkanError> {
    let shader_dir = env::current_exe()?.parent().unwrap().join(path.into());

    Ok(shader_dir)
  }

  fn cached_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, VulkanError> {
    let shader_cache_dir = env::current_exe()?
      .parent()
      .context("invalid exe parent")?
      .join(ShaderStore::SHADER_CACHE_DIR);

    let mut cached_path = shader_cache_dir.join(
      path
        .as_ref()
        .strip_prefix(ShaderStore::SHADER_ASSET_DIR)
        .unwrap_or_else(|_error| path.as_ref()),
    );
    
    // let file_name = cached_path.file_name().context("invalid file name")?;
    let parent = cached_path.parent().context("invalid file parent")?;

    info!("{:?}", cached_path);
    if let Err(error) = fs::create_dir_all(parent) {
      error!("{error}");
    };

    let mut extension = cached_path.extension().context("invalid file extension")?.to_os_string();
    extension.push(".spv");

    cached_path.set_extension(extension);

    Ok(cached_path)
  }

  fn cached_file_younger_than_exe<P: AsRef<Path>>(cached_file: P) -> Result<bool, VulkanError> {
    let file_age = cached_file.as_ref().metadata()?.modified()?;
    let exe_age = env::current_exe()?.metadata()?.modified()?;
    // debug!("File age: [{file_age:?}], Exe age: [{exe_age:?}]");
    Ok(file_age >= exe_age)
  }

  fn compile_shader_type<S: StageInfo, P: AsRef<Path>>(source: &str, output_path: &P) -> Result<Vec<u8>, VulkanError> {
    let output_path = output_path.as_ref();
    let compiler = shaderc::Compiler::new().context("failed to initialize shaderc compiler")?;
    let mut options = shaderc::CompileOptions::new().context("failed to initialize shaderc compiler options")?;

    options.set_source_language(shaderc::SourceLanguage::GLSL);

    if !cfg!(debug_assertions) {
      options.set_optimization_level(shaderc::OptimizationLevel::Performance);
    }

    match compiler.compile_into_spirv(
      source,
      S::kind().into(),
      output_path
        .file_name()
        .context("failed to access file_name")?
        .to_str()
        .context("failed to convert file_name to str")?,
      S::kind().entry_point().as_str(),
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
      Err(err) => Err(VulkanError::Shaderc(err)),
    }
  }
}

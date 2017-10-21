
use std::error::Error;
use std::fmt;

pub use gfx_device_gl::Version as GlslVersion;

/// Shader backend with version numbers.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Backend {
    Glsl(GlslVersion),
    GlslEs(GlslVersion),
}

pub trait ShadeExt {
    fn shader_backend(&self) -> Backend;
}

impl ShadeExt for ::gfx_device_gl::Device {
    fn shader_backend(&self) -> Backend {
        let shade_lang = self.get_info().shading_language;
        if shade_lang.is_embedded {
            Backend::GlslEs(shade_lang)
        } else {
            Backend::Glsl(shade_lang)
        }
    }
}

pub const EMPTY: &'static [u8] = &[];

/// A type storing shader source for different graphics APIs and versions.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Source<'a> {
    pub glsl_120: &'a [u8],
    pub glsl_130: &'a [u8],
    pub glsl_140: &'a [u8],
    pub glsl_150: &'a [u8],
    pub glsl_330: &'a [u8],
    pub glsl_400: &'a [u8],
    pub glsl_430: &'a [u8],
    pub glsl_es_100: &'a [u8],
    pub glsl_es_200: &'a [u8],
    pub glsl_es_300: &'a [u8],
    pub hlsl_30: &'a [u8],
    pub hlsl_40: &'a [u8],
    pub hlsl_41: &'a [u8],
    pub hlsl_50: &'a [u8],
    pub msl_10: &'a [u8],
    pub msl_11: &'a [u8],
    pub vulkan: &'a [u8],
}

/// Error selecting a backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectError(Backend);

impl fmt::Display for SelectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occurred when selecting the {:?} backend", self.0)
    }
}

impl Error for SelectError {
    fn description(&self) -> &str {
        "An error occurred when selecting a backend"
    }
}

impl<'a> Source<'a> {
    /// Create an empty shader source. Useful for specifying the remaining
    /// structure members upon construction.
    pub fn empty() -> Source<'a> {
        Source {
            glsl_120: EMPTY,
            glsl_130: EMPTY,
            glsl_140: EMPTY,
            glsl_150: EMPTY,
            glsl_330: EMPTY,
            glsl_400: EMPTY,
            glsl_430: EMPTY,
            glsl_es_100: EMPTY,
            glsl_es_200: EMPTY,
            glsl_es_300: EMPTY,
            hlsl_30: EMPTY,
            hlsl_40: EMPTY,
            hlsl_41: EMPTY,
            hlsl_50: EMPTY,
            msl_10: EMPTY,
            msl_11: EMPTY,
            vulkan: EMPTY,
        }
    }

    /// Pick one of the stored versions that is the highest supported by the backend.
    pub fn select(&self, backend: Backend) -> Result<&'a [u8], SelectError> {
        Ok(match backend {
            Backend::Glsl(version) => {
                let v = version.major * 100 + version.minor;
                match *self {
                    Source { glsl_430: s, .. } if s != EMPTY && v >= 430 => s,
                    Source { glsl_400: s, .. } if s != EMPTY && v >= 400 => s,
                    Source { glsl_330: s, .. } if s != EMPTY && v >= 300 => s,
                    Source { glsl_150: s, .. } if s != EMPTY && v >= 150 => s,
                    Source { glsl_140: s, .. } if s != EMPTY && v >= 140 => s,
                    Source { glsl_130: s, .. } if s != EMPTY && v >= 130 => s,
                    Source { glsl_120: s, .. } if s != EMPTY && v >= 120 => s,
                    _ => return Err(SelectError(backend)),
                }
            }
            Backend::GlslEs(version) => {
                let v = version.major * 100 + version.minor;
                match *self {
                    Source { glsl_es_100: s, .. } if s != EMPTY && v >= 100 => s,
                    Source { glsl_es_200: s, .. } if s != EMPTY && v >= 200 => s,
                    Source { glsl_es_300: s, .. } if s != EMPTY && v >= 300 => s,
                    _ => return Err(SelectError(backend)),
                }
            }
        })
    }
}

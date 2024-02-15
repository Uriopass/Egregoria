use mlua::{FromLua, Value};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderAsset {
    Mesh { path: PathBuf },
    Sprite { path: PathBuf },
}

impl RenderAsset {
    pub fn is_mesh(&self) -> bool {
        matches!(self, Self::Mesh { .. })
    }

    pub fn is_sprite(&self) -> bool {
        matches!(self, Self::Sprite { .. })
    }
}

impl<'lua> FromLua<'lua> for RenderAsset {
    fn from_lua(value: Value<'lua>, _lua: &'lua mlua::Lua) -> mlua::Result<Self> {
        match value {
            Value::String(path) => {
                let path = path.to_string_lossy().to_string();

                let Some(ext) = path.rsplit_once('.').map(|(_, ext)| ext) else {
                    return Err(mlua::Error::external(format!(
                        "Asset path {} has no extension",
                        path
                    )));
                };

                match ext {
                    "glb" => Ok(Self::Mesh {
                        path: PathBuf::from(path),
                    }),
                    "png" | "jpg" => Ok(Self::Sprite {
                        path: PathBuf::from(path),
                    }),
                    _ => Err(mlua::Error::external(format!(
                        "Unknown asset extension: {}",
                        ext
                    ))),
                }
            }
            Value::Table(t) => {
                let path = t.get::<_, String>("path")?;

                let Some(ext) = path.rsplit_once('.').map(|(_, ext)| ext) else {
                    return Err(mlua::Error::external(format!(
                        "Asset path {} has no extension",
                        path
                    )));
                };

                match ext {
                    "glb" => Ok(Self::Mesh {
                        path: PathBuf::from(path),
                    }),
                    "png" | "jpg" => Ok(Self::Sprite {
                        path: PathBuf::from(path),
                    }),
                    _ => Err(mlua::Error::external(format!(
                        "Unknown asset extension: {}",
                        ext
                    ))),
                }
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "AssetDefinition",
                message: Some("expected a string or a table".into()),
            }),
        }
    }
}

impl std::fmt::Display for RenderAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mesh { path } => write!(f, "Mesh({})", path.display()),
            Self::Sprite { path } => write!(f, "Sprite({})", path.display()),
        }
    }
}

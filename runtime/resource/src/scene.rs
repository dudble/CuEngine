use std::path::PathBuf;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use platform::{file_service, path};

const SCENE_FILE_NAME: &str = "scene.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LevelRes {
    #[serde(default)]
    pub objects: Vec<ObjectRes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRes {
    pub name: String,
    #[serde(default)]
    pub position: [f32; 2],
    #[serde(default)]
    pub components: Vec<ComponentInstanceRes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInstanceRes {
    pub type_name: String,
    #[serde(default)]
    pub data: Option<toml::Value>,
}

fn load_path() -> Option<PathBuf> {
    let cwd_candidate = PathBuf::from(SCENE_FILE_NAME);
    if file_service::exists(&cwd_candidate) {
        return Some(cwd_candidate);
    }

    let exe_candidate = path::join(&path::executable_dir(), SCENE_FILE_NAME);
    if file_service::exists(&exe_candidate) {
        return Some(exe_candidate);
    }

    None
}

pub fn load<T: DeserializeOwned>() -> Option<T> {
    let scene_path = load_path()?;

    match file_service::read_to_string(&scene_path) {
        Ok(text) => match toml::from_str::<T>(&text) {
            Ok(data) => {
                engine_core::log_info!("scene loaded from {}", scene_path.display());
                Some(data)
            }
            Err(error) => {
                engine_core::log_error!("failed to parse {}: {error}", scene_path.display());
                None
            }
        },
        Err(error) => {
            engine_core::log_error!("failed to read {}: {error}", scene_path.display());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_level() {
        let text = r#"
[[objects]]
name = "player"
position = [1.0, 2.0]

[[objects.components]]
type_name = "mover"
data = { velocity = [3.0, 0.0], min_x = -4.0, max_x = 4.0 }
"#;

        let level: LevelRes = toml::from_str(text).expect("parse");

        assert_eq!(level.objects.len(), 1);
        assert_eq!(level.objects[0].name, "player");
        assert_eq!(level.objects[0].position, [1.0, 2.0]);
        assert_eq!(level.objects[0].components[0].type_name, "mover");
        assert!(level.objects[0].components[0].data.is_some());
    }
}

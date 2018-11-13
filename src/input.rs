use std::fs::File;
use std::collections::HashMap;
use serde_json;


enum Action {
    MoveCameraLeft,
    MoveCameraRight,
    MoveCameraUp,
    MoveCameraDown,
    MoveCameraForward,
    MoveCameraBackwards,
    YawCameraLeft,
    YawCameraRight,
    PitchCameraUp,
    PitchCameraDown,
    RollCameraLeft,
    RollCameraRight,
    ExitProgram,
    ResetCameraPosition,
    DoNothing,
}

enum Key {
    A,
    D,
    Q,
    E,
    W,
    S,
    LeftArrow,
    RightArrow,
    UpArrow,
    DownArrow,
    Z,
    C,
    Escape,
    Backspace,
}

type KeyMap = HashMap<Action, Key>;

enum Error {
    FileNotFound(String),
    CouldNotParseKeyConfigurationFile(String),
}

fn load_json(file: &str) -> Result<HashMap<String, String>, serde_json::Error> {
    let data = File::open("src/controls.json").expect("File not found.");
    let key_map = serde_json::from_reader(data)?;

    Ok(key_map)
}


fn load(file: &str) -> Result<KeyMap, Error> {
    let key_map = load_json(file).unwrap();

    Ok(key_map)
}


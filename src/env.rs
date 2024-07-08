pub const DEFAULT_DOTENV_FILE_CONTENTS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/.default.env"));
pub const DEFAULT_DOTENV_FILENAME: &str = ".env";


pub fn load_dotenv() {
    log::debug!("loading dotenv file...");
    
    match dotenvy::dotenv() {
        Ok(_) => (),
        Err(dotenvy::Error::Io(io_err)) if io_err.kind() == std::io::ErrorKind::NotFound => {
            create_default_dotenv();
            load_dotenv();
        },
        Err(err ) => panic!("unhandled error occurred during dotenv file loading: {err}")
    }
}


fn create_default_dotenv() {
    log::debug!("creating default dotenv file...");
    
    std::fs::write(DEFAULT_DOTENV_FILENAME, DEFAULT_DOTENV_FILE_CONTENTS)
        .unwrap_or_else(|err| panic!("an error occurred during creation of default dotenv file: {err}"));
}

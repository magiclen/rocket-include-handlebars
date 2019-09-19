use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::handlebars::TemplateFileError;
use crate::Handlebars;

#[derive(Debug)]
/// Reloadable Handlebars.
pub struct ReloadableHandlebars {
    handlebars: Handlebars,
    files: HashMap<&'static str, (PathBuf, Option<SystemTime>)>,
}

impl ReloadableHandlebars {
    #[cfg(feature = "helper")]
    #[inline]
    /// Create an instance of `ReloadableHandlebars`.
    pub fn new() -> ReloadableHandlebars {
        let mut handlebars = Handlebars::new();

        handlebars_helpers!(handlebars);

        ReloadableHandlebars {
            handlebars,
            files: HashMap::new(),
        }
    }

    #[cfg(not(feature = "helper"))]
    #[inline]
    /// Create an instance of `ReloadableHandlebars`.
    pub fn new() -> ReloadableHandlebars {
        ReloadableHandlebars {
            handlebars: Handlebars::new(),
            files: HashMap::new(),
        }
    }

    #[inline]
    /// Register a template from a path and it can be reloaded automatically.
    pub fn register_template_file<P: Into<PathBuf>>(
        &mut self,
        name: &'static str,
        file_path: P,
    ) -> Result<(), TemplateFileError> {
        let file_path = file_path.into();

        let metadata = file_path
            .metadata()
            .map_err(|err| TemplateFileError::IOError(err, name.to_string()))?;

        let mtime = metadata.modified().ok();

        self.handlebars.register_template_file(name, &file_path)?;

        self.files.insert(name, (file_path, mtime));

        Ok(())
    }

    #[inline]
    /// Unregister a template from a file by a name.
    pub fn unregister_template_file<S: AsRef<str>>(&mut self, name: S) -> Option<PathBuf> {
        let name = name.as_ref();

        match self.files.remove(name) {
            Some((file_path, _)) => {
                self.handlebars.unregister_template(name);

                Some(file_path)
            }
            None => None,
        }
    }

    #[inline]
    /// Reload templates if needed.
    pub fn reload_if_needed(&mut self) -> Result<(), TemplateFileError> {
        for (name, (file_path, mtime)) in &mut self.files {
            let metadata = file_path
                .metadata()
                .map_err(|err| TemplateFileError::IOError(err, name.to_string()))?;

            let (reload, new_mtime) = match mtime {
                Some(mtime) => {
                    match metadata.modified() {
                        Ok(new_mtime) => (new_mtime > *mtime, Some(new_mtime)),
                        Err(_) => (true, None),
                    }
                }
                None => {
                    match metadata.modified() {
                        Ok(new_mtime) => (true, Some(new_mtime)),
                        Err(_) => (true, None),
                    }
                }
            };

            if reload {
                self.handlebars.register_template_file(name, &file_path)?;

                *mtime = new_mtime;
            }
        }

        Ok(())
    }
}

impl Default for ReloadableHandlebars {
    #[inline]
    fn default() -> Self {
        ReloadableHandlebars::new()
    }
}

impl Deref for ReloadableHandlebars {
    type Target = Handlebars;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.handlebars
    }
}

impl DerefMut for ReloadableHandlebars {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.handlebars
    }
}

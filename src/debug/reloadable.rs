use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::functions::add_helpers;
use crate::handlebars::{Handlebars, TemplateError};

#[derive(Debug)]
/// Reloadable Handlebars.
pub struct ReloadableHandlebars {
    handlebars: Handlebars<'static>,
    files: HashMap<&'static str, (PathBuf, Option<SystemTime>)>,
}

impl ReloadableHandlebars {
    /// Create an instance of `ReloadableHandlebars`.
    #[inline]
    pub fn new() -> ReloadableHandlebars {
        let mut handlebars = Handlebars::new();

        add_helpers(&mut handlebars);

        ReloadableHandlebars {
            handlebars,
            files: HashMap::new(),
        }
    }

    /// Register a template from a path and it can be reloaded automatically.
    #[inline]
    pub fn register_template_file<P: Into<PathBuf>>(
        &mut self,
        name: &'static str,
        file_path: P,
    ) -> Result<(), TemplateError> {
        let file_path = file_path.into();

        let metadata =
            file_path.metadata().map_err(|err| TemplateError::from((err, String::from(name))))?;

        let mtime = metadata.modified().ok();

        self.handlebars.register_template_file(name, &file_path)?;

        self.files.insert(name, (file_path, mtime));

        Ok(())
    }

    /// Unregister a template from a file by a name.
    #[inline]
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

    /// Reload templates if needed.
    #[inline]
    pub fn reload_if_needed(&mut self) -> Result<(), TemplateError> {
        for (&name, (file_path, mtime)) in &mut self.files {
            let metadata = file_path
                .metadata()
                .map_err(|err| TemplateError::from((err, String::from(name))))?;

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
    type Target = Handlebars<'static>;

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

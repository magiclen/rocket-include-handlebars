use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::SystemTime,
};

use handlebars::{Handlebars, TemplateError};

use crate::functions::add_helpers;

#[derive(Debug)]
/// Reloadable Handlebars.
pub struct ReloadableHandlebars {
    handlebars: Handlebars<'static>,
    files:      HashMap<&'static str, (PathBuf, Option<SystemTime>)>,
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
    ) -> Result<(), Box<TemplateError>> {
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
            },
            None => None,
        }
    }

    /// Reload templates if needed.
    ///
    /// A template that fails to reload does not stop the others from being reloaded, and the first error is returned.
    #[inline]
    pub fn reload_if_needed(&mut self) -> Result<(), Box<TemplateError>> {
        let mut first_error = None;

        for (&name, (file_path, mtime)) in &mut self.files {
            let new_mtime = match file_path.metadata() {
                Ok(metadata) => metadata.modified().ok(),
                Err(err) => {
                    first_error
                        .get_or_insert(Box::new(TemplateError::from((err, String::from(name)))));

                    continue;
                },
            };

            if new_mtime.is_none() || new_mtime != *mtime {
                match self.handlebars.register_template_file(name, file_path) {
                    Ok(()) => *mtime = new_mtime,
                    Err(err) => {
                        first_error.get_or_insert(Box::new(err));
                    },
                }
            }
        }

        first_error.map_or(Ok(()), Err)
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

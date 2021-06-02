//! Directory Tree Simulator: Provides a directory tree structure and an operating system stub
//! structure to interact with it.

// Bart Massey 2021

// Workaround for Clippy false positive in Rust 1.51.0.
// https://github.com/rust-lang/rust-clippy/issues/6546
#![allow(clippy::result_unit_err)]

use thiserror::Error;

/// Errors during directory interaction.
#[derive(Error, Debug)]
pub enum DirError<'a> {
    /// The character `/` in component names is disallowed,
    /// to make path separators easier.
    #[error("{0}: slash in name is invalid")]
    SlashInName(&'a str),
    /// Only one subdirectory of a given name can exist in any directory.
    #[error("{0}: directory exists")]
    DirExists(&'a str),
    /// Traversal failed due to missing subdirectory.
    #[error("{0}: invalid element in path")]
    InvalidChild(&'a str),
}

/// Result type for directory errors.
pub type Result<'a, T> = std::result::Result<T, DirError<'a>>;

/// A directory entry. Component names are stored externally.
#[derive(Debug, Clone)]
pub struct DEnt<'a> {
    pub name: &'a str,
    pub subdir: DTree<'a>,
}

/// A directory tree.
#[derive(Debug, Clone, Default)]
pub struct DTree<'a> {
    pub children: Vec<DEnt<'a>>,
}

/// Operating system state: the directory tree and the current working directory.
#[derive(Debug, Clone, Default)]
pub struct OsState<'a> {
    pub dtree: DTree<'a>,
    pub cwd: Vec<&'a str>,
}

impl<'a> DEnt<'a> {
    pub fn new(name: &'a str) -> Result<Self> {
        Ok(DEnt { 
            name, 
            subdir:DTree::new(),    
        })
    }
}

impl<'a> DTree<'a> {
    /// Create a new empty directory tree.
    pub fn new() -> Self {
        Self::default()
    }

    /// Make a subdirectory with the given name in this directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dtree::DTree;
    /// let mut dt = DTree::new();
    /// dt.mkdir("test").unwrap();
    /// assert_eq!(&dt.paths(), &["/test/"]);
    /// ```
    ///
    /// # Errors
    ///
    /// * `DirError::SlashInName` if `name` contains `/`.
    /// * `DirError::DirExists` if `name` already exists.
    pub fn mkdir(&mut self, name: &'a str) -> Result<()> {
        if name.contains("/"){ return Err(DirError::SlashInName("{0}: slash in name is invalid"));}
        let d: DEnt<'a> = DEnt::new(name).unwrap();
        let mut found: bool = false;
        for n in &self.children{
           if n.name.eq(name){found = true;}
        }
        match found {
            true => Err(DirError::DirExists("{0}: directory exists")),
            false => {
                self.children.push(d);
                Ok(())
            },
        }
    }

    /// Traverse to the subdirectory given by `path` and then call `f` to visit the subdirectory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dtree::DTree;
    /// let mut dt = DTree::new();
    /// dt.mkdir("test").unwrap();
    /// let paths = dt.with_subdir(&["test"], |dt| dt.paths()).unwrap();
    /// assert_eq!(&paths, &["/"]);
    /// ```
    ///
    /// # Errors
    ///
    /// * `DirError::InvalidChild` if `path` is invalid.
    pub fn with_subdir<'b, F, R>(&'b self, path: &[&'a str], f: F) -> Result<R>
    where
        F: FnOnce(&'b DTree<'a>) -> R,
    {
        for p in path{
            for d in &self.children{
                if d.name == p.to_string(){
                    return Ok(f(&d.subdir));
                }
            }
        }
        Err(DirError::InvalidChild("Invalid Child"))
    }

    /// Traverse to the subdirectory given by `path` and then call `f` to visit the subdirectory
    /// mutably.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dtree::DTree;
    /// let mut dt = DTree::new();
    /// dt.mkdir("a").unwrap();
    /// dt.with_subdir_mut(&["a"], |dt| dt.mkdir("b").unwrap()).unwrap();
    /// assert_eq!(&dt.paths(), &["/a/b/"]);
    /// ```
    ///
    /// # Errors
    ///
    /// * `DirError::InvalidChild` if `path` is invalid.
    pub fn with_subdir_mut<'b, F, R>(&'b mut self, path: &[&'a str], f: F) -> Result<R>
    where
        F: FnOnce(&'b mut DTree<'a>) -> R,
    {
        for p in path{
            self.find_child(p);
            return Ok(f(self));
        }
        Err(DirError::InvalidChild("Invalid child in with sub dir"))
    }

    fn find_child<'b>(&'b self, p: &&str) -> &'b DTree<'a>{
        for d in &self.children{
            if p.to_string() == d.name{
                return &d.subdir;
            }
        }
        panic!("Invalid child")
    }
    /// Produce a list of the paths to each reachable leaf, in no particular order.  Path
    /// components are prefixed by `/`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dtree::DTree;
    /// let mut dt = DTree::new();
    /// dt.mkdir("a").unwrap();
    /// dt.with_subdir_mut(&["a"], |dt| dt.mkdir("b").unwrap()).unwrap();
    /// dt.with_subdir_mut(&["a"], |dt| dt.mkdir("c").unwrap()).unwrap();
    /// let mut paths = dt.paths();
    /// paths.sort();
    /// assert_eq!(&paths, &["/a/b/", "/a/c/"]);
    /// ```
  
    pub fn paths(&self) -> Vec<String> {
        let mut retpaths: Vec<String> = Vec::new();
        if self.children.is_empty(){
            retpaths.push("/".to_string())
        }
        for n in &self.children {
            retpaths.push(format!("/{}{}", n.name, n.subdir.path_helper()));
        }
        retpaths
    }
    
    fn path_helper(&self) -> String{
        let mut cwd: String = String::new();
        if self.children.is_empty(){return "/".to_string();}
        for z in &self.children{
            cwd = format!("/{}{}", z.name, z.subdir.path_helper())
       }
        cwd
    }
}

impl<'a> OsState<'a> {
    /// Create a new directory tree in the operating system.  Current working directory is the
    /// root.
    pub fn new() -> Self {
        Self::default()
    }

    /// If `path` is empty, change the working directory to the root.  Otherwise change the
    /// working directory to the subdirectory given by `path` relative to the current working
    /// directory.  (There is no notion of `.` or `..`: `path` must be a valid sequence of
    /// component names.)
    ///
    /// # Examples
    ///
    /// ```
    /// # use dtree::OsState;
    /// let mut s = OsState::new();
    /// s.mkdir("a").unwrap();
    /// s.chdir(&["a"]).unwrap();
    /// s.mkdir("b").unwrap();
    /// s.chdir(&["b"]).unwrap();
    /// s.mkdir("c").unwrap();
    /// s.chdir(&[]).unwrap();
    /// assert_eq!(&s.paths().unwrap(), &["/a/b/c/"]);
    /// ```
    ///
    /// # Errors
    ///
    /// * `DirError::InvalidChild` if the new working directory is invalid. On error, the original
    /// working directory will be retained.
    pub fn chdir(&mut self, path: &[&'a str]) -> Result<()> {
        let mut x: DTree<'a> = DTree::new();
        for p in path{
            for child in &self.dtree.children{
                if p.to_string() == child.name{
                    x = child.subdir.clone();
                }
            }
        }
        self.dtree = x.clone();
        Ok(())
    }

    /// Make a new subdirectory with the given `name` in the working directory.
    ///
    /// # Errors
    ///
    /// * `DirError::SlashInName` if `name` contains `/`.
    /// * `DirError::InvalidChild` if the current working directory is invalid.
    /// * `DirError::DirExists` if `name` already exists.
    pub fn mkdir(&mut self, name: &'a str) -> Result<()> {
        if name.contains("/"){return Err(DirError::SlashInName("Slash in name"))}
        else{}
        let d: DEnt<'a> = DEnt::new(name).unwrap();
        let mut found: bool = false;

        for n in &self.dtree.children{
            if n.name.eq(name){found=true;}
        }
        match found{
            true => Err(DirError::DirExists("Directory exists")),
            false => {
                self.dtree.children.push(d);
                Ok(())
            },
        }
    }

    /// Produce a list of the paths from the working directory to each reachable leaf, in no
    /// particular order.  Path components are separated by `/`.
    ///
    /// # Errors
    ///
    /// * `DirError::InvalidChild` if the current working directory is invalid.
    pub fn paths(&self) -> Result<Vec<String>> {
        let mut retpaths: Vec<String> = Vec::new();
        if self.dtree.children.is_empty(){
            retpaths.push("/".to_string())
        }
        for n in &self.dtree.children{
            retpaths.push(format!("/{}{}", n.name, n.subdir.path_helper()));
        }
        match retpaths.is_empty(){
            true => Ok(retpaths),
            _ => Err(DirError::InvalidChild("Invalid child in paths")),
        }
    }
}

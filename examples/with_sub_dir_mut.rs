fn main(){
    let mut dt = dtree::DTree::new();
    dt.mkdir("test").unwrap();
    dt.with_subdir_mut(&["test"], |dt| dt.mkdir("test2").unwrap()).unwrap();
    dt.with_subdir_mut(&["test"], |dt| dt.mkdir("test3").unwrap()).unwrap();
    println!("{:?}", &dt.paths());
}
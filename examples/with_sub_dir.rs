fn main(){
    let mut dt = dtree::DTree::new();
    dt.mkdir("test").unwrap();
    dt.children[0].subdir.mkdir("test2").unwrap();
    let paths = dt.with_subdir(&["test"], |dt|dt.paths()).unwrap();
    println!("{:?}", paths);
    println!("{:?}",dt.paths());
}
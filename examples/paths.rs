fn main(){
    let mut dt = dtree::DTree::new();
    dt.mkdir("Marks Test Directory").unwrap();
    dt.children[0].subdir.mkdir("Marks Other Directory").unwrap();
    dt.mkdir("Crystals Test Directory").unwrap();
    let paths1 = dt.paths();
    println!("{:?}", paths1);
}
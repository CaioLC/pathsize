use std::{error::Error, collections::HashMap};
use clap::Parser;
use walkdir::WalkDir;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Walk through directories and sort by total size")]
struct Args {
    path: String
}

struct Node {
    path: String,
    size: u64,
    children: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let dir_root = WalkDir::new(args.path.clone());
    let mut full_tree = HashMap::<usize, Vec<Node>>::new();
    for entry in dir_root {
        let dir_entry = entry?;
        let depth = dir_entry.depth();
        // println!("{depth} -- {}", dir_entry.path().display());

        if dir_entry.file_type().is_dir() {
            let node = Node {
                path: String::from(dir_entry.path().to_string_lossy()),
                size: 0,
                children: 0
            };

            match full_tree.get_mut(&depth){
                Some(vec_node) => vec_node.push(node),
                None => {
                    full_tree.insert(depth, vec![node]);
                },
            }
            if depth >= 1 {
                let vec_node = full_tree.get_mut(&(depth - 1)).unwrap();
                vec_node.last_mut().expect("failed to access vector in a previous node").children += 1;
            }
        } else {
            for i in 0..depth {
                let node_vec = full_tree.get_mut(&i).expect(&format!("expected node dir at depth {i}"));
                let mut node = node_vec.last_mut().expect("vector should be initialized already");
                node.size += dir_entry.metadata()?.len();
            }
        }
    }
    for i in 0..3 {
        let node_vec = full_tree.get(&i).expect(&format!("expected node dir at depth {i}"));
        for node in node_vec {
            if node.size > GB {
                println!("{} -- {} GB | {} subfolders", node.path, node.size as f32 / GB as f32, node.children);
            } else if node.size > MB {
                println!("{} -- {} MB | {} subfolders", node.path, node.size as f32 / MB as f32, node.children);
            } else {
                println!("{} -- {} KB | {} subfolders", node.path, node.size as f32 / KB as f32, node.children);
            }
        }
    }

    // let sorted = sort_hash(&full_tree);
    Ok(())
}

// fn sort_hash(full_tree: &HashMap<usize, Vec<Node>>) -> _ {
//     todo!()
// }

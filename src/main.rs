use std::{error::Error, collections::HashMap};
use clap::Parser;
use walkdir::WalkDir;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Walk through directories and sort by total size")]
struct Args {
    path: String,
    #[clap(short, long, default_value = "4")]
    max_depth: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Node {
    path: String,
    depth: usize,
    size: u64,
    has_children: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let dir_root = WalkDir::new(args.path.clone());

    // Traverse directory tree, depth first.
    let mut full_tree = build_tree(dir_root)?;
    // Sort each tree depth by size.
    sort_tree(&mut full_tree);
    // for each level, take the corresponding children directory and return structure as a vector
    let flatten = flat_map(&full_tree, 0, &args.path);
    // custom print for the final vector structure.
    print_tree(flatten, args.max_depth);
    Ok(())
}

fn build_tree(dir_root: WalkDir) -> Result<HashMap::<usize, Vec<Node>>, Box<dyn Error>> {
    println!("Traversing directories...");
    let mut full_tree = HashMap::<usize, Vec<Node>>::new();
    for entry in dir_root {
        let dir_entry = entry?;
        let depth = dir_entry.depth();
        // println!("{depth} -- {}", dir_entry.path().display());

        if dir_entry.file_type().is_dir() {
            let node = Node {
                path: String::from(dir_entry.path().to_string_lossy()),
                depth,
                size: 0,
                has_children: false
            };

            match full_tree.get_mut(&depth){
                Some(vec_node) => vec_node.push(node),
                None => {
                    full_tree.insert(depth, vec![node]);
                },
            }
            if depth >= 1 {
                let vec_node = full_tree.get_mut(&(depth - 1)).unwrap();
                vec_node.last_mut().expect("failed to access vector in a previous node").has_children = true;
            }
        } else {
            for i in 0..depth {
                let node_vec = full_tree.get_mut(&i).expect(&format!("expected node dir at depth {i}"));
                let mut node = node_vec.last_mut().expect("vector should be initialized already");
                node.size += dir_entry.metadata()?.len();
            }
        }
    }
    Ok(full_tree)
}

fn print_tree(flatten: Vec<Node>, max_depth: usize) {
    for node in flatten.into_iter().filter(|v| v.depth <= max_depth) {
        let dashes = node.depth;
        if node.size > GB {
            print!("|{: <width$}", "", width = dashes*2);
            println!("{} -- {} GB", node.path, node.size as f32 / GB as f32);
        } else if node.size > MB {
            print!("|{: <width$}", "", width = dashes*2);
            println!("{} -- {} MB", node.path, node.size as f32 / MB as f32);
        } else {
            print!("|{: <width$}", "", width = dashes*2);
            println!("{} -- {} KB", node.path, node.size as f32 / KB as f32);
        }
    }
}

fn sort_tree(full_tree: &mut HashMap<usize, Vec<Node>>) {
    for depth in 0..full_tree.len() {
        let vec_node = full_tree.get_mut(&depth).unwrap();
        vec_node.sort_by(|a, b| b.size.cmp(&a.size));
    }
}

fn flat_map(sorted_three: &HashMap<usize, Vec<Node>>, depth: usize, pattern: &String) -> Vec<Node> {
    let mut flat_vec = Vec::new();
    let vec_nodes: Vec<&Node> = sorted_three
        .get(&depth)
        .expect(format!("Expected key at depth {}", depth).as_str())
        .iter()
        .filter(|n| n.path.contains(pattern))
        .collect();
    for node in vec_nodes {
        flat_vec.push(node.clone());
        if node.has_children {
            let children = flat_map(sorted_three, depth+1, &node.path);
            for child in children {
                flat_vec.push(child);
            }
        }; 
    }
    flat_vec
}

use std::{env, fs, io::Write, path::PathBuf};

   fn main() {
       // Get the output directory where the generated code will be stored
       let out_dir = PathBuf::from(".");

       // Read the directory contents
       let mut files = fs::read_dir("./sprites")
        .unwrap().filter(|entry|{
            entry.as_ref().unwrap().file_name().clone().into_string().unwrap().ends_with(".png")
        })
        .map(|entry| entry.unwrap().file_name().into_string().unwrap().replace(".png",""))
        .collect::<Vec<_>>();

        // Sort the files with a custom comparator
        files.sort_by(|a, b| {
            let a_is_digit = a.chars().next().unwrap().is_digit(10);
            let b_is_digit = b.chars().next().unwrap().is_digit(10);
            
            match (a_is_digit, b_is_digit) {
                // Both start with digits, so compare numerically
                (true, true) => a.cmp(b),
                // Only 'a' starts with a digit, so it should come first
                (true, false) => std::cmp::Ordering::Less,
                // Only 'b' starts with a digit, so it should come first
                (false, true) => std::cmp::Ordering::Greater,
                // Both don't start with digits, so compare alphabetically
                (false, false) => a.cmp(b),
            }
        });

       // Create the Rust code with a fixed-size array
       let array_code = format!(
           "pub const FILE_NAMES: [&str; {}] = {:?};",
           files.len(),
           files,
       );

       // Write the generated code to a file in the output directory
       let dest_path = out_dir.join("src/file_list.rs");
       if let Ok(file) = fs::read(dest_path.clone()){
            if file == array_code.as_bytes(){
                return
            }
       }
       let mut f = fs::File::create(&dest_path).unwrap();
       f.write_all(array_code.as_bytes()).unwrap();
   }
use clap::Parser;
use std::path::PathBuf;

mod categories;
mod organizer;
mod scanner;

use categories::CategoryMapper;
use organizer::Organizer;
use scanner::DirectoryScanner;

/// A CLI tool to organize files by their extensions into categorized folders
#[derive(Parser, Debug)]
#[command(name = "file-organizer")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Source directory containing files to organize
    #[arg(short, long, default_value = ".")]
    pub source: PathBuf,

    /// Output directory for organized files (defaults to source directory)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Preview changes without actually moving files
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Show verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

fn main() {
    let args = Args::parse();

    println!("File Organizer");
    println!("==============");
    println!("Source directory: {}", args.source.display());

    let output_dir = args.output.as_ref().unwrap_or(&args.source);
    println!("Output directory: {}", output_dir.display());

    if args.dry_run {
        println!("Mode: Dry run (no files will be moved)");
    }

    if args.verbose {
        println!("Verbose mode: enabled");
    }

    let mapper = CategoryMapper::new();

    if args.verbose {
        println!("\nSupported categories:");
        for category in mapper.all_categories() {
            println!("  - {}", category.folder_name());
        }
    }

    // Scan the source directory
    println!("\nScanning directory...");
    let scanner = DirectoryScanner::new(mapper);

    match scanner.scan(&args.source) {
        Ok(result) => {
            println!("Found {} files to organize\n", result.total_count);

            if result.total_count == 0 {
                println!("No files to organize.");
                return;
            }

            if args.verbose {
                println!("Files by category:");
                for category in CategoryMapper::new().all_categories() {
                    let count = result.category_count(&category);
                    if count > 0 {
                        println!("  {}: {} file(s)", category.folder_name(), count);
                        if let Some(files) = result.categorized.get(&category) {
                            for file in files {
                                println!("    - {}", file.name);
                            }
                        }
                    }
                }
            } else {
                println!("Files by category:");
                for category in CategoryMapper::new().all_categories() {
                    let count = result.category_count(&category);
                    if count > 0 {
                        println!("  {}: {} file(s)", category.folder_name(), count);
                    }
                }
            }

            // Create directory structure
            println!("\nCreating directory structure...");
            let organizer = Organizer::new(output_dir.clone(), args.dry_run, args.verbose);

            match organizer.create_category_directories(&result) {
                Ok(created) => {
                    if args.dry_run {
                        println!("Would create {} directories", created.len());
                    } else if created.is_empty() {
                        println!("All directories already exist");
                    } else {
                        println!("Created {} directories", created.len());
                    }
                }
                Err(e) => {
                    eprintln!("Error creating directories: {}", e);
                    std::process::exit(1);
                }
            }

            // Move files to their categories
            println!("\nMoving files...");
            let move_result = organizer.move_files(&result);

            // Print final summary
            move_result.print_summary(args.dry_run);
        }
        Err(e) => {
            eprintln!("Error scanning directory: {}", e);
            std::process::exit(1);
        }
    }
}

use std::error::Error;

#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
const ONNX_RELEASE_URL: &str = "https://github.com/microsoft/onnxruntime/releases/download/v1.16.3/onnxruntime-linux-x64-1.16.3.tgz";
#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
const LIBTORCH_RELEASE_URL: &str =
    "https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.4.0%2Bcpu.zip";

#[cfg(not(feature = "runtime-env"))]
fn main() -> Result<(), Box<dyn Error>> {
    use std::{env, process::Command};
    dotenvy::dotenv().expect("Failed to read .env file. Did you `cp .env.dist .env` ?");

    #[cfg(feature = "hallucination-detection")]
    #[cfg(feature = "ner")]
    copy_shared_objects()?;

    let output = Command::new("npx")
        .arg("tailwindcss")
        .arg("-i")
        .arg("./static/in.css")
        .arg("-o")
        .arg("./static/output.css")
        .output()?;

    // Stream output
    println!("{}", String::from_utf8_lossy(&output.stdout));

    for (key, value) in env::vars() {
        println!("cargo:rustc-env={key}={value}");
    }

    println!("cargo:rerun-if-changed=.env");

    minijinja_embed::embed_templates!("src/public");
    Ok(())
}

#[cfg(feature = "runtime-env")]
fn main() -> Result<(), Box<dyn Error>> {
    minijinja_embed::embed_templates!("src/public");

    #[cfg(feature = "hallucination-detection")]
    #[cfg(feature = "ner")]
    copy_shared_objects()?;

    Ok(())
}

#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
fn download_file<P>(url: String, target_file: &P)
where
    P: AsRef<Path>,
{
    let resp = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(300))
        .call()
        .unwrap_or_else(|err| panic!("ERROR: Failed to download {}: {:?}", url, err));

    let len = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap();
    let mut reader = resp.into_reader();
    // FIXME: Save directly to the file
    let mut buffer = vec![];
    let read_len = reader.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer.len(), len);
    assert_eq!(buffer.len(), read_len);

    let f = fs::File::create(target_file).unwrap();
    let mut writer = io::BufWriter::new(f);
    writer.write_all(&buffer).unwrap();
}

#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
fn copy_shared_objects() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:rustc-link-search={}", out_dir.display());

    let libonnxfilezip = out_dir.join("libonnxruntime.tgz");
    let libonnxfile = out_dir.join("onnxruntime-linux-x64-1.16.3/lib/libonnxruntime.so");
    let libtorchfilezip = out_dir.join("libtorch.zip");
    let libtorchfile = out_dir.join("libtorch");

    if !libonnxfilezip.as_path().exists() {
        download_file(ONNX_RELEASE_URL.to_string(), &libonnxfilezip);
    }

    if !libtorchfilezip.as_path().exists() {
        download_file(LIBTORCH_RELEASE_URL.to_string(), &libtorchfilezip);
    }

    if !libonnxfile.as_path().exists() {
        extract_tgz(&libonnxfilezip, out_dir.as_path());
    }

    if !libtorchfile.as_path().exists() {
        extract_zip(&libtorchfilezip, out_dir.as_path());
    }

    env::set_var("ORT_DYLIB_PATH", libonnxfile.display().to_string());

    env::set_var("LIBTORCH", libtorchfile.display().to_string());
    env::set_var(
        "LD_LIBRARY_PATH",
        libtorchfile.join("lib").display().to_string(),
    );

    Ok(())
}

#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
fn extract_tgz(filename: &Path, output: &Path) {
    let file = fs::File::open(filename).unwrap();
    let buf = io::BufReader::new(file);
    let tar = flate2::read::GzDecoder::new(buf);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(output).unwrap();
}

#[cfg(feature = "hallucination-detection")]
#[cfg(feature = "ner")]
fn extract_zip(filename: &Path, output: &Path) {
    let file = fs::File::open(filename).unwrap_or_else(|err| {
        panic!(
            "ERROR: Failed to open file {}: {:?}",
            filename.display(),
            err
        )
    });
    let mut archive = zip::ZipArchive::new(file).unwrap_or_else(|err| {
        panic!(
            "ERROR: Failed to open zip archive {}: {:?}",
            filename.display(),
            err
        )
    });
    archive.extract(output).unwrap_or_else(|err| {
        panic!(
            "ERROR: Failed to extract zip archive {}: {:?}",
            filename.display(),
            err
        )
    });
}

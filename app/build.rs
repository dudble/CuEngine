
fn main(){
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-arg=/STACK:8388608");// 增大栈容量
    }
}
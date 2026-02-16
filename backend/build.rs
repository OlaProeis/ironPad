fn main() {
    // On Windows, embed the application icon into the .exe
    // This sets both the Explorer icon and makes it available as a resource
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/ironpad.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}

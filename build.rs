fn current_year() -> u32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Rough estimation of the year since Unix Epoch (1 year = 31,556,952 seconds)
    1970 + (secs / 31_556_952) as u32
}

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set("CompanyName", "Takuma WATANABE");

        // Dynamically combine start year and current year for copyright display
        let start_year = 2026;
        let this_year = current_year();
        let copyright = if this_year > start_year {
            format!("(C) {}-{} Takuma WATANABE", start_year, this_year)
        } else {
            format!("(C) {} Takuma WATANABE", start_year)
        };
        res.set("LegalCopyright", &copyright);

        res.set("ProductName", "openssh-shim-wsl2");
        if let Err(e) = res.compile() {
            eprintln!("warning: failed to compile Windows resources: {e}");
        }
    }
}

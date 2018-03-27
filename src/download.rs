pub struct DownloadArguments<'a> {
    pub project_name: &'a str,
    pub branch_name: &'a str,
    pub key: &'a str,
    pub url: &'a str,
    pub cache: &'a str,
}


pub fn download<'a>(args: DownloadArguments<'a>) -> Result<(), String> {
    Ok(())
}
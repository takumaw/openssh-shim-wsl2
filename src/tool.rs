#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tool {
    Ssh,
    Scp,
    Sftp,
}

impl Tool {
    pub fn command(self) -> &'static str {
        match self {
            Self::Ssh => "ssh",
            Self::Scp => "scp",
            Self::Sftp => "sftp",
        }
    }
}

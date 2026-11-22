use std::path::PathBuf;

/// Erros da biblioteca local.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("erro de banco de dados: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("erro de E/S em {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("não foi possível determinar o diretório de dados do usuário")]
    NoDataDir,
    #[error("{0} não encontrado")]
    NotFound(&'static str),
    #[error("versão do banco ({found}) é mais nova que a suportada ({supported})")]
    SchemaTooNew { found: i64, supported: i64 },
    #[error("posição inválida: {0}")]
    InvalidPosition(usize),
    #[error("faixa remota inválida no banco: {0}")]
    InvalidRemote(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

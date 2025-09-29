use common::types::BoxError;
use tantivy::{IndexWriter, TantivyDocument, Term};
use tokio::sync::{mpsc, oneshot};

#[allow(dead_code)]
#[derive(Clone)]
pub(crate) struct IndexWriterHandle {
    sender: mpsc::Sender<Command>,
}

#[allow(dead_code)]
enum Command {
    AddDocument {
        doc: TantivyDocument,
        respond: oneshot::Sender<Result<(), BoxError>>,
    },
    DeleteTerm {
        term: Term,
        respond: oneshot::Sender<Result<u64, BoxError>>,
    },
    Commit {
        respond: oneshot::Sender<Result<(), BoxError>>,
    },
}

#[allow(dead_code)]
impl IndexWriterHandle {
    // add document
    pub(crate) async fn add_document(&self, doc: TantivyDocument) -> Result<(), BoxError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Command::AddDocument { doc, respond: tx })
            .await?;
        rx.await?
    }

    // delete trem
    pub(crate) async fn delete_term(&self, term: Term) -> Result<u64, BoxError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Command::DeleteTerm { term, respond: tx })
            .await?;
        rx.await?
    }

    // commit
    pub(crate) async fn commit(&self) -> Result<(), BoxError> {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Command::Commit { respond: tx }).await?;
        rx.await?
    }
}

#[allow(dead_code)]
pub(crate) fn spawn_index_writer_task(mut writer: IndexWriter) -> IndexWriterHandle {
    let (tx, mut rx) = mpsc::channel::<Command>(100);

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Command::AddDocument { doc, respond } => {
                    let res = (|| -> Result<(), BoxError> {
                        writer.add_document(doc)?;
                        tracing::debug!("add document !!!");
                        Ok(())
                    })();
                    let _ = respond.send(res);
                }
                Command::DeleteTerm { term, respond } => {
                    let res = (|| -> Result<u64, BoxError> {
                        writer.delete_term(term);
                        tracing::debug!("delete term !!!");
                        Ok(0)
                    })();
                    let _ = respond.send(res);
                }
                Command::Commit { respond } => {
                    let res = (|| -> Result<(), BoxError> {
                        writer.commit()?;
                        tracing::debug!("commit !!!");
                        Ok(())
                    })();
                    let _ = respond.send(res);
                }
            }
        }
    });

    IndexWriterHandle { sender: tx }
}

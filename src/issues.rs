use super::{Error, Jira, Result, Issue};

/// issue options
#[derive(Debug)]
pub struct Issues {
    jira: Jira,
}

#[derive(Serialize, Debug)]
pub struct AssignIssue {
    name: String,
}

impl Issues {
    pub fn new(jira: &Jira) -> Issues {
        Issues { jira: jira.clone() }
    }

    pub fn get<I>(&self, id: I) -> Result<Issue>
    where
        I: Into<String>,
    {
        self.jira.get(&format!("/issue/{}", id.into()))
    }

    pub fn assign<I>(&self, id: I, name: I) -> Result<()> where I: Into<String> {
        let assign = AssignIssue{name: name.into()};
        self.jira
            .put::<(), AssignIssue>(
                &format!("/issue/{}/assignee", id.into()),
                assign,
            )
            .or_else(|e| match e {
                Error::Serde(_) => Ok(()),
                e => Err(e),
            })
    }
}

use crate::{
    parse::{self, ParseError},
    print,
};
use alloc::borrow::Cow;
use core::fmt::{Debug, Display, Error as FmtError, Formatter, Result as FmtResult, Write};
use hop::{
    backend::{memory::Error as MemoryError, Backend},
    request::CommandConfigurationError,
    Client,
};
use hop_engine::command::{CommandId, DispatchError, Request};
use std::error::Error;

#[derive(Debug)]
pub enum ProcessError<B: Backend>
where
    B::Error: Error,
{
    Backend { source: <B as Backend>::Error },
    ParsingInput { source: ParseError },
    WritingOutput { source: FmtError },
}

impl<B: Backend> Display for ProcessError<B>
where
    B::Error: Error + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Backend { source } => {
                f.write_fmt(format_args!("backend encountered an error: {}", source))
            }
            Self::ParsingInput { source } => {
                f.write_fmt(format_args!("failed to parse input: {}", source))
            }
            Self::WritingOutput { source } => {
                f.write_fmt(format_args!("failed to write output: {}", source))
            }
        }
    }
}

impl<B: Backend + Debug + 'static> Error for ProcessError<B>
where
    B::Error: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Backend { source } => Some(source),
            Self::ParsingInput { source } => Some(source),
            Self::WritingOutput { source } => Some(source),
        }
    }
}

impl<B: Backend> From<ParseError> for ProcessError<B>
where
    B::Error: Error,
{
    fn from(source: ParseError) -> Self {
        Self::ParsingInput { source }
    }
}

impl<B: Backend> From<FmtError> for ProcessError<B>
where
    B::Error: Error,
{
    fn from(source: FmtError) -> Self {
        Self::WritingOutput { source }
    }
}

enum InnerProcessError<B: Backend> {
    Backend { source: <B as Backend>::Error },
    BadRequest,
    BuildingRequest,
    Dispatching,
    KeyDestinationRequired,
    KeyNonexistent,
    KeyRequiredMinimum,
    KeySourceRequired,
    KeyTypeDifferent,
    KeyTypeInvalid,
    KeyTypeRequired,
    KeyTypeUnexpected,
    KeyUnspecified,
    PreconditionFailed,
    TooFewArguments,
    TooManyArguments,
    WritingOutput { source: FmtError },
}

impl<B: Backend> From<FmtError> for InnerProcessError<B> {
    fn from(source: FmtError) -> Self {
        Self::WritingOutput { source }
    }
}

impl<B: Backend> From<CommandConfigurationError> for InnerProcessError<B> {
    fn from(source: CommandConfigurationError) -> Self {
        match source {
            CommandConfigurationError::NoKeys => Self::KeyRequiredMinimum,
            CommandConfigurationError::TooManyKeys => Self::TooManyArguments,
        }
    }
}

pub async fn process<B: Backend + Send + Sync + 'static>(
    client: &Client<B>,
    input: &str,
) -> Result<Cow<'static, str>, ProcessError<B>>
where
    B::Error: Error,
{
    let req = parse::parse(input)?;

    Ok(match process_inner(client, req).await {
        Ok(output) => output,
        Err(InnerProcessError::Backend { source }) => return Err(ProcessError::Backend { source }),
        Err(InnerProcessError::BadRequest) => {
            "The server rejected the request due to being invalid.".into()
        }
        Err(InnerProcessError::BuildingRequest) => {
            "Building the request failed, such as due to too many arguments.".into()
        }
        Err(InnerProcessError::Dispatching) => {
            "The engine failed to properly run the request.".into()
        }
        Err(InnerProcessError::KeyDestinationRequired) => {
            "The destination key name is required.".into()
        }
        Err(InnerProcessError::KeyNonexistent) => "The specified key does not exist.".into(),
        Err(InnerProcessError::KeyRequiredMinimum) => {
            "A minimum of at least one key or more is required.".into()
        }
        Err(InnerProcessError::KeySourceRequired) => "The source key name is required.".into(),
        Err(InnerProcessError::KeyTypeDifferent) => {
            "The type of the key is different than specified by the command.".into()
        }
        Err(InnerProcessError::KeyTypeInvalid) => {
            "A key type was provided that isn't supported by the command.".into()
        }
        Err(InnerProcessError::KeyTypeRequired) => {
            "A key type was required but one was not specified.".into()
        }
        Err(InnerProcessError::KeyTypeUnexpected) => {
            "A key type was specified when the command can't be given one.".into()
        }
        Err(InnerProcessError::KeyUnspecified) => "Specifying a key is required.".into(),
        Err(InnerProcessError::PreconditionFailed) => {
            "A precondition failed, such as the key not existing.".into()
        }
        Err(InnerProcessError::TooFewArguments) => {
            "Too few arguments were provided for this command.".into()
        }
        Err(InnerProcessError::TooManyArguments) => {
            "You may only provide at most 255 arguments.".into()
        }
        Err(InnerProcessError::WritingOutput { source }) => {
            format!("Failed to write the response: {}", source).into()
        }
    })
}

fn backend_err<B: Backend>(err: <B as Backend>::Error) -> InnerProcessError<B>
where
    B::Error: Error + 'static,
{
    let b: Box<dyn Error> = Box::new(err);

    let err = match b.downcast::<MemoryError>() {
        Ok(memory_error) => {
            return match *memory_error {
                MemoryError::BadRequest { .. } => InnerProcessError::BadRequest,
                MemoryError::BuildingRequest { .. } => InnerProcessError::BuildingRequest,
                MemoryError::Dispatching { .. } => InnerProcessError::Dispatching,
                MemoryError::KeyTypeInvalid { .. } => InnerProcessError::KeyTypeInvalid,
                MemoryError::KeyTypeUnsupported { .. } => InnerProcessError::KeyTypeInvalid,
                MemoryError::RunningCommand { source } => match source {
                    DispatchError::ArgumentRetrieval => InnerProcessError::TooFewArguments,
                    DispatchError::KeyNonexistent => InnerProcessError::KeyNonexistent,
                    DispatchError::KeyTypeDifferent => InnerProcessError::KeyTypeDifferent,
                    DispatchError::KeyTypeInvalid => InnerProcessError::KeyTypeInvalid,
                    DispatchError::KeyTypeRequired => InnerProcessError::KeyTypeRequired,
                    DispatchError::KeyTypeUnexpected => InnerProcessError::KeyTypeUnexpected,
                    DispatchError::KeyUnspecified => InnerProcessError::KeyUnspecified,
                    DispatchError::PreconditionFailed => InnerProcessError::PreconditionFailed,
                },
            }
        }
        Err(err) => err,
    };

    InnerProcessError::Backend {
        source: *err
            .downcast::<B::Error>()
            .expect("error must be same as provided"),
    }
}

async fn process_inner<B: Backend + Send + Sync + 'static>(
    client: &Client<B>,
    req: Request<'_>,
) -> Result<Cow<'static, str>, InnerProcessError<B>>
where
    B::Error: Error,
{
    match req.command_id() {
        CommandId::Decrement => {
            let key = req.key().ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let v = client.decrement(key).await.map_err(backend_err)?;

            Ok(print::value(v).into())
        }
        CommandId::Delete => {
            let key = req.key().ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let v = client.delete(key).await.map_err(backend_err)?;

            Ok(String::from_utf8_lossy(&v).into_owned().into())
        }
        CommandId::Echo => {
            if let Some(req_args) = req.args(..) {
                let req_args = req_args.collect::<Vec<_>>().join(b" ".as_ref());
                let args = client.echo(req_args).await.map_err(backend_err)?;

                let output = args
                    .into_iter()
                    .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                    .collect::<Vec<String>>()
                    .join(" ");

                Ok(output.into())
            } else {
                Ok("".into())
            }
        }
        CommandId::Exists => {
            let args = req
                .args(..)
                .ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let req = match client.exists().keys(args) {
                Ok(req) => req,
                Err(CommandConfigurationError::NoKeys) => {
                    return Err(InnerProcessError::KeyRequiredMinimum)
                }
                Err(CommandConfigurationError::TooManyKeys) => {
                    return Err(InnerProcessError::TooManyArguments)
                }
            };

            let exists = req.await.map_err(backend_err)?;

            Ok(exists.to_string().into())
        }
        CommandId::Get => {
            let key = req.key().ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let value = client.get(key).await.map_err(backend_err)?;

            Ok(print::value(value).into())
        }
        CommandId::Increment => {
            let key = req.key().ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let v = client.increment(key).await.map_err(backend_err)?;

            Ok(print::value(v).into())
        }
        CommandId::Is => {
            let key_type = req
                .key_type()
                .ok_or_else(|| InnerProcessError::KeyTypeRequired)?;
            let args = req
                .args(..)
                .ok_or_else(|| InnerProcessError::TooFewArguments)?
                .collect::<Vec<_>>();

            let is_type = client.is(key_type).keys(args)?.await.map_err(backend_err)?;

            Ok(is_type.to_string().into())
        }
        CommandId::Keys => {
            let key = req.key().ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let v = client.keys(key).await.map_err(backend_err)?;

            Ok(print_list(v).into())
        }
        CommandId::Rename => {
            let from = req
                .key()
                .ok_or_else(|| InnerProcessError::KeyDestinationRequired)?;
            let to = req
                .arg(1)
                .ok_or_else(|| InnerProcessError::KeySourceRequired)?;

            let v = client.rename(from, to).await.map_err(backend_err)?;

            Ok(String::from_utf8_lossy(&v).into_owned().into())
        }
        CommandId::Stats => {
            let stats = client.stats().await.map_err(backend_err)?;

            let mut output = String::new();
            writeln!(
                output,
                "Commands successful: {}",
                stats.commands_successful()
            )
            .map_err(|source| InnerProcessError::WritingOutput { source })?;
            writeln!(output, "Commands errored: {}", stats.commands_errored())
                .map_err(|source| InnerProcessError::WritingOutput { source })?;
            writeln!(output, "Sessions started: {}", stats.sessions_started())
                .map_err(|source| InnerProcessError::WritingOutput { source })?;
            write!(output, "Sessions ended: {}", stats.sessions_ended())
                .map_err(|source| InnerProcessError::WritingOutput { source })?;

            Ok(output.into())
        }
        CommandId::Type => {
            let key = req.key().ok_or_else(|| InnerProcessError::KeyUnspecified)?;

            let key_type = client.key_type(key).await.map_err(backend_err)?;

            Ok(print::key_type_name(key_type).into())
        }
        _ => panic!(),
    }
}

fn print_list(list: Vec<Vec<u8>>) -> String {
    list.into_iter()
        .map(|item| String::from_utf8_lossy(&item).into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

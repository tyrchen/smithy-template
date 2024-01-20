#[macro_export]
macro_rules! err {
    ($ty:ident, $msg:expr) => {
        Err(echo_server_sdk::error::ServerError {
            code: echo_server_sdk::model::ErrorCode::$ty,
            message: $msg.to_string(),
        }.into())
    };
    ($ty:ident, $msg:expr, $($param:expr),*) => {
        Err(echo_server_sdk::error::ServerError {
            code: echo_server_sdk::model::ErrorCode::$ty,
            message: format!($msg, $($param),*),
        })
    };
}

#[macro_export]
macro_rules! bail {
    ($ty:ident, $msg:expr) => {
        return $crate::err!($ty, $msg)
    };
    ($ty:ident, $msg:expr, $($param:expr),*) => {
        return $crate::err!($ty, $msg, $($param),*)
    };
}

#[macro_export]
macro_rules! try_err {
    ($expr:expr, $ty:ident) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return $crate::err!($ty, e.to_string()),
        }
    };
    ($expr:expr, $ty:ident, $msg:expr) => {
        match $expr {
            Some(v) => v,
            None => return $crate::err!($ty, $msg),
        }
    };
}

#[macro_export]
macro_rules! not_found {
    ( $msg:expr) => {
        return Err(echo_server_sdk::error::NotFoundError {
            message: $msg.to_string(),
        }.into())
    };
    ($msg:expr, $($param:expr),*) => {
        return Err(echo_server_sdk::error::NotFoundError {
            message: format!($msg, $($param),*),
        }.into())
    };
}

#[macro_export]
macro_rules! conflict {
    ( $msg:expr) => {
        return Err(echo_server_sdk::error::ConflictError {
            message: $msg.to_string(),
        }.into())
    };
    ($msg:expr, $($param:expr),*) => {
        return Err(echo_server_sdk::error::ConflictError {
            message: format!($msg, $($param),*),
        }.into())
    };
}

#[macro_export]
macro_rules! forbidden {
    ( $msg:expr) => {
        return Err(echo_server_sdk::error::ForbiddenError {
            message: $msg.to_string(),
        }.into())
    };
    ($msg:expr, $($param:expr),*) => {
        return Err(echo_server_sdk::error::ForbiddenError {
            message: format!($msg, $($param),*),
        }.into())
    };
}

#[macro_export]
macro_rules! unauthorized {
    ( $msg:expr) => {
        return Err(echo_server_sdk::error::UnauthorizedError {
            message: $msg.to_string(),
        }.into())
    };
    ($msg:expr, $($param:expr),*) => {
        return Err(echo_server_sdk::error::UnauthorizedError {
            message: format!($msg, $($param),*),
        }.into())
    };
}

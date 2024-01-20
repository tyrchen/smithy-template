$version: "2.0"

namespace com.example

use aws.protocols#restJson1
use smithy.framework#ValidationException
use aws.api#service


/// Echoes input
@service(sdkId: "echo")
@restJson1
@httpBearerAuth
service EchoService {
    version: "2023-12-03"
    operations: [EchoMessage, Signin]
}

@http(uri: "/echo", method: "POST")
@auth([])
operation EchoMessage {
    input := {
        @required
        @httpHeader("x-echo-message")
        message: String
    }
    output := {
        @required
        message: String
    }
    errors: [ValidationException]
}


/// Signin to get a token.
@http(uri: "/signin", method: "POST")
@auth([])
operation Signin {
    input := {
        @required
        username: String
        @required
        password: String
    }
    output := {
        @required
        token: String
    }
    errors: [ValidationException, UnauthorizedError, ForbiddenError, ThrottlingError]
}

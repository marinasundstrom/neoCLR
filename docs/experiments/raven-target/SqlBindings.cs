// Documentation/compiler-reference declarations only. Native libraries own executable bodies.
// Union cases are projected from the actual Raven source by SourceUnionReferences.
static class SqlBindings
{
    public const string Declarations = """
namespace Data.Sql {
public struct SqlError {
    public struct InvalidInput { } public struct InvalidState { } public struct Transport { }
    public struct Protocol { } public struct Unsupported { } public struct LimitExceeded { }
    public struct Server { } public struct Mapping { }
}
public struct SqlValue {
    public struct Null { } public struct Text { } public struct Integer { }
    // Avoid primitive-bootstrap text rewriting of the unrelated nested case name.
    public partial struct Boolean { }
}
public interface SqlDatabase { Tasks.Task<Result<SqlConnection, SqlError>> Open(); }
public interface SqlConnection : Disposable {
    Tasks.Task<Result<SqlResult, SqlError>> Execute(SqlStatement statement);
    Tasks.Task<Result<SqlTransaction, SqlError>> Begin();
}
public interface SqlTransaction : Disposable {
    Tasks.Task<Result<PropagationUnit, SqlError>> Commit();
    Tasks.Task<Result<PropagationUnit, SqlError>> Rollback();
}
public class SqlStatement {
    public SqlStatement(string text) { }
    public SqlStatement(string text, Collections.Sequence<SqlValue> values) { }
    public string Text => default;
    public int ParameterCount => default;
    public SqlValue Parameter(int index) => default;
}
public class SqlRow {
    public SqlRow(Collections.Sequence<string> columns, Collections.Sequence<Option<string>> cells) { }
    public int Count => default;
    public Result<string, SqlError> Column(int index) => default;
    public Result<Option<string>, SqlError> Text(int index) => default;
    public Result<Option<string>, SqlError> Text(string name) => default;
    public Result<string, SqlError> RequiredText(string name) => default;
    public Result<int, SqlError> Int32(string name) => default;
    public Result<long, SqlError> Int64(string name) => default;
    public Result<bool, SqlError> Boolean(string name) => default;
}
public class SqlResult {
    public SqlResult(string commandTag, Collections.Sequence<SqlRow> values) { }
    public int Count => default;
    public string CommandTag => default;
    public SqlRow Row(int index) => default;
}
public class SqlMapper {
    private SqlMapper() { }
    public static Result<Collections.Sequence<T>, SqlError> Map<T>(SqlResult result) => default;
    public static Result<Collections.Sequence<T>, SqlError> Map<T>(SqlResult result, Func<SqlRow, Result<T, SqlError>> map) => default;
}
}
namespace Data.PostgreSql {
public class PostgresDatabase : Sql.SqlDatabase {
    public PostgresDatabase(string database, string user, int port, int timeoutMilliseconds) { }
    public Tasks.Task<Result<Sql.SqlConnection, Sql.SqlError>> Open() => default;
}
}
""";
}

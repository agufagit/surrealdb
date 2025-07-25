use crate::{
	sql::{
		Algorithm, Array, Base, Block, Cond, Data, Datetime, Dir, Duration, Edges, Explain,
		Expression, Fetch, Fetchs, Field, Fields, Future, Graph, Group, Groups, Id, Ident, Idiom,
		Idioms, Index, Kind, Limit, Number, Object, Operator, Order, Output, Param, Part,
		Permission, Permissions, Scoring, Split, Splits, SqlValue, SqlValues, Start, Statement,
		Strand, Subquery, Table, TableType, Tables, Thing, Timeout, Uuid, Version, With,
		access::AccessDuration,
		access_type::{
			AccessType, BearerAccess, BearerAccessSubject, BearerAccessType, JwtAccess,
			JwtAccessIssue, JwtAccessVerify, JwtAccessVerifyJwks, JwtAccessVerifyKey, RecordAccess,
		},
		block::Entry,
		changefeed::ChangeFeed,
		filter::Filter,
		graph::{GraphSubject, GraphSubjects},
		index::{Distance, HnswParams, MTreeParams, SearchParams, VectorType},
		language::Language,
		order::{OrderList, Ordering},
		statements::{
			AccessStatement, BeginStatement, BreakStatement, CancelStatement, CommitStatement,
			ContinueStatement, CreateStatement, DefineAccessStatement, DefineAnalyzerStatement,
			DefineDatabaseStatement, DefineEventStatement, DefineFieldStatement,
			DefineFunctionStatement, DefineIndexStatement, DefineNamespaceStatement,
			DefineParamStatement, DefineStatement, DefineTableStatement, DeleteStatement,
			ForeachStatement, IfelseStatement, InfoStatement, InsertStatement, KillStatement,
			OptionStatement, OutputStatement, RelateStatement, RemoveAccessStatement,
			RemoveAnalyzerStatement, RemoveDatabaseStatement, RemoveEventStatement,
			RemoveFieldStatement, RemoveFunctionStatement, RemoveIndexStatement,
			RemoveNamespaceStatement, RemoveParamStatement, RemoveStatement, RemoveTableStatement,
			RemoveUserStatement, SelectStatement, ThrowStatement, UpdateStatement, UpsertStatement,
			UseStatement,
			access::{
				self, AccessStatementGrant, AccessStatementPurge, AccessStatementRevoke,
				AccessStatementShow,
			},
			analyze::AnalyzeStatement,
			show::{ShowSince, ShowStatement},
			sleep::SleepStatement,
		},
		tokenizer::Tokenizer,
		user::UserDuration,
	},
	syn::parser::{
		ParserSettings,
		mac::{test_parse, test_parse_with_settings},
	},
};
use chrono::{NaiveDate, Offset, Utc, offset::TimeZone};

fn ident_field(name: &str) -> SqlValue {
	SqlValue::Idiom(Idiom(vec![Part::Field(Ident(name.to_string()))]))
}

#[test]
pub fn parse_analyze() {
	let res = test_parse!(parse_stmt, r#"ANALYZE INDEX b on a"#).unwrap();
	assert_eq!(
		res,
		Statement::Analyze(AnalyzeStatement::Idx(Ident("a".to_string()), Ident("b".to_string())))
	)
}

#[test]
pub fn parse_begin() {
	let res = test_parse!(parse_stmt, r#"BEGIN"#).unwrap();
	assert_eq!(res, Statement::Begin(BeginStatement));
	let res = test_parse!(parse_stmt, r#"BEGIN TRANSACTION"#).unwrap();
	assert_eq!(res, Statement::Begin(BeginStatement));
}

#[test]
pub fn parse_break() {
	let res = test_parse!(parse_stmt, r#"BREAK"#).unwrap();
	assert_eq!(res, Statement::Break(BreakStatement));
}

#[test]
pub fn parse_cancel() {
	let res = test_parse!(parse_stmt, r#"CANCEL"#).unwrap();
	assert_eq!(res, Statement::Cancel(CancelStatement));
	let res = test_parse!(parse_stmt, r#"CANCEL TRANSACTION"#).unwrap();
	assert_eq!(res, Statement::Cancel(CancelStatement));
}

#[test]
pub fn parse_commit() {
	let res = test_parse!(parse_stmt, r#"COMMIT"#).unwrap();
	assert_eq!(res, Statement::Commit(CommitStatement));
	let res = test_parse!(parse_stmt, r#"COMMIT TRANSACTION"#).unwrap();
	assert_eq!(res, Statement::Commit(CommitStatement));
}

#[test]
pub fn parse_continue() {
	let res = test_parse!(parse_stmt, r#"CONTINUE"#).unwrap();
	assert_eq!(res, Statement::Continue(ContinueStatement));
}

#[test]
fn parse_create() {
	let res = test_parse!(
		parse_stmt,
		"CREATE ONLY foo SET bar = 3, foo +?= baz RETURN VALUE foo AS bar TIMEOUT 1s PARALLEL"
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Create(CreateStatement {
			only: true,
			what: SqlValues(vec![SqlValue::Table(Table("foo".to_owned()))]),
			data: Some(Data::SetExpression(vec![
				(
					Idiom(vec![Part::Field(Ident("bar".to_owned()))]),
					Operator::Equal,
					SqlValue::Number(Number::Int(3))
				),
				(
					Idiom(vec![Part::Field(Ident("foo".to_owned()))]),
					Operator::Ext,
					SqlValue::Idiom(Idiom(vec![Part::Field(Ident("baz".to_owned()))]))
				),
			])),
			output: Some(Output::Fields(Fields(
				vec![Field::Single {
					expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident("foo".to_owned()))])),
					alias: Some(Idiom(vec![Part::Field(Ident("bar".to_owned()))])),
				}],
				true,
			))),
			timeout: Some(Timeout(Duration(std::time::Duration::from_secs(1)))),
			parallel: true,
			version: None,
		}),
	);
}

#[test]
fn parse_define_namespace() {
	let res = test_parse!(parse_stmt, "DEFINE NAMESPACE a COMMENT 'test'").unwrap();
	assert_eq!(
		res,
		Statement::Define(DefineStatement::Namespace(DefineNamespaceStatement {
			id: None,
			name: Ident("a".to_string()),
			comment: Some(Strand("test".to_string())),
			if_not_exists: false,
			overwrite: false,
		}))
	);

	let res = test_parse!(parse_stmt, "DEFINE NS a").unwrap();
	assert_eq!(
		res,
		Statement::Define(DefineStatement::Namespace(DefineNamespaceStatement {
			id: None,
			name: Ident("a".to_string()),
			comment: None,
			if_not_exists: false,
			overwrite: false,
		}))
	)
}

#[test]
fn parse_define_database() {
	let res =
		test_parse!(parse_stmt, "DEFINE DATABASE a COMMENT 'test' CHANGEFEED 10m INCLUDE ORIGINAL")
			.unwrap();
	assert_eq!(
		res,
		Statement::Define(DefineStatement::Database(DefineDatabaseStatement {
			id: None,
			name: Ident("a".to_string()),
			comment: Some(Strand("test".to_string())),
			changefeed: Some(ChangeFeed {
				expiry: std::time::Duration::from_secs(60) * 10,
				store_diff: true,
			}),
			if_not_exists: false,
			overwrite: false,
		}))
	);

	let res = test_parse!(parse_stmt, "DEFINE DB a").unwrap();
	assert_eq!(
		res,
		Statement::Define(DefineStatement::Database(DefineDatabaseStatement {
			id: None,
			name: Ident("a".to_string()),
			comment: None,
			changefeed: None,
			if_not_exists: false,
			overwrite: false,
		}))
	)
}

#[test]
fn parse_define_function() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE FUNCTION fn::foo::bar($a: number, $b: array<bool,3>) {
			RETURN a
		} COMMENT 'test' PERMISSIONS FULL
		"#
	)
	.unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Function(DefineFunctionStatement {
			name: Ident("foo::bar".to_string()),
			args: vec![
				(Ident("a".to_string()), Kind::Number),
				(Ident("b".to_string()), Kind::Array(Box::new(Kind::Bool), Some(3)))
			],
			block: Block(vec![Entry::Output(OutputStatement {
				what: ident_field("a"),
				fetch: None,
			})]),
			comment: Some(Strand("test".to_string())),
			permissions: Permission::Full,
			if_not_exists: false,
			overwrite: false,
			returns: None,
		}))
	)
}

#[test]
fn parse_define_user() {
	// Password.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSWORD 'hunter2' COMMENT "*******""#
		)
		.unwrap();

		let Statement::Define(DefineStatement::User(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("user".to_string()));
		assert_eq!(stmt.base, Base::Root);
		assert!(stmt.hash.starts_with("$argon2id$"));
		assert_eq!(stmt.roles, vec![Ident("Viewer".to_string())]);
		assert_eq!(stmt.comment, Some(Strand("*******".to_string())));
		assert_eq!(
			stmt.duration,
			UserDuration {
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
	}
	// Passhash.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' COMMENT "*******""#
		)
		.unwrap();

		let Statement::Define(DefineStatement::User(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("user".to_string()));
		assert_eq!(stmt.base, Base::Root);
		assert_eq!(stmt.hash, "hunter2".to_owned());
		assert_eq!(stmt.roles, vec![Ident("Viewer".to_string())]);
		assert_eq!(stmt.comment, Some(Strand("*******".to_string())));
		assert_eq!(
			stmt.duration,
			UserDuration {
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
	}
	// With roles.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' ROLES editor, OWNER"#
		)
		.unwrap();

		let Statement::Define(DefineStatement::User(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("user".to_string()));
		assert_eq!(stmt.base, Base::Root);
		assert_eq!(stmt.hash, "hunter2".to_owned());
		assert_eq!(stmt.roles, vec![Ident("editor".to_string()), Ident("OWNER".to_string())]);
		assert_eq!(
			stmt.duration,
			UserDuration {
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
	}
	// With session duration.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' DURATION FOR SESSION 6h"#
		)
		.unwrap();

		let Statement::Define(DefineStatement::User(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("user".to_string()));
		assert_eq!(stmt.base, Base::Root);
		assert_eq!(stmt.hash, "hunter2".to_owned());
		assert_eq!(stmt.roles, vec![Ident("Viewer".to_string())]);
		assert_eq!(
			stmt.duration,
			UserDuration {
				token: Some(Duration::from_hours(1).unwrap()),
				session: Some(Duration::from_hours(6).unwrap()),
			}
		);
	}
	// With session and token duration.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' DURATION FOR TOKEN 15m, FOR SESSION 6h"#
		)
		.unwrap();

		let Statement::Define(DefineStatement::User(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("user".to_string()));
		assert_eq!(stmt.base, Base::Root);
		assert_eq!(stmt.hash, "hunter2".to_owned());
		assert_eq!(stmt.roles, vec![Ident("Viewer".to_string())]);
		assert_eq!(
			stmt.duration,
			UserDuration {
				token: Some(Duration::from_mins(15).unwrap()),
				session: Some(Duration::from_hours(6).unwrap()),
			}
		);
	}
	// With none token duration.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' DURATION FOR TOKEN NONE"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of user with none token duration: {:?}",
			res
		);
	}
	// With nonexistent role.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' ROLES foo"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of user with nonexistent role: {:?}",
			res
		);
	}
	// With existent and nonexistent roles.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE USER user ON ROOT COMMENT 'test' PASSHASH 'hunter2' ROLES Viewer, foo"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of user with nonexistent role: {:?}",
			res
		);
	}
}

// TODO(gguillemas): This test is kept in 2.0.0 for backward compatibility. Drop in 3.0.0.
#[test]
fn parse_define_token() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE TOKEN a ON DATABASE TYPE EDDSA VALUE "foo" COMMENT "bar""#
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Define(DefineStatement::Access(DefineAccessStatement {
			name: Ident("a".to_string()),
			base: Base::Db,
			kind: AccessType::Jwt(JwtAccess {
				verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
					alg: Algorithm::EdDSA,
					key: "foo".to_string(),
				}),
				issue: None,
			}),
			authenticate: None,
			// Default durations.
			duration: AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			},
			comment: Some(Strand("bar".to_string())),
			if_not_exists: false,
			overwrite: false,
		})),
	)
}

// TODO(gguillemas): This test is kept in 2.0.0 for backward compatibility. Drop in 3.0.0.
#[test]
fn parse_define_token_on_scope() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE TOKEN a ON SCOPE b TYPE EDDSA VALUE "foo" COMMENT "bar""#
	)
	.unwrap();

	// Manually compare since DefineAccessStatement for record access
	// without explicit JWT will create a random signing key during parsing.
	let Statement::Define(DefineStatement::Access(stmt)) = res else {
		panic!()
	};

	assert_eq!(stmt.name, Ident("a".to_string()));
	assert_eq!(stmt.base, Base::Db); // Scope base is ignored.
	assert_eq!(
		stmt.duration,
		// Default durations.
		AccessDuration {
			grant: Some(Duration::from_days(30).unwrap()),
			token: Some(Duration::from_hours(1).unwrap()),
			session: None,
		}
	);
	assert_eq!(stmt.comment, Some(Strand("bar".to_string())));
	assert!(!stmt.if_not_exists);
	match stmt.kind {
		AccessType::Record(ac) => {
			assert_eq!(ac.signup, None);
			assert_eq!(ac.signin, None);
			match ac.jwt.verify {
				JwtAccessVerify::Key(key) => {
					assert_eq!(key.alg, Algorithm::EdDSA);
				}
				_ => panic!(),
			}
			assert_eq!(ac.jwt.issue, None);
		}
		_ => panic!(),
	}
}

// TODO(gguillemas): This test is kept in 2.0.0 for backward compatibility. Drop in 3.0.0.
#[test]
fn parse_define_token_jwks() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE TOKEN a ON DATABASE TYPE JWKS VALUE "http://example.com/.well-known/jwks.json" COMMENT "bar""#
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Define(DefineStatement::Access(DefineAccessStatement {
			name: Ident("a".to_string()),
			base: Base::Db,
			kind: AccessType::Jwt(JwtAccess {
				verify: JwtAccessVerify::Jwks(JwtAccessVerifyJwks {
					url: "http://example.com/.well-known/jwks.json".to_string(),
				}),
				issue: None,
			}),
			authenticate: None,
			// Default durations.
			duration: AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			},
			comment: Some(Strand("bar".to_string())),
			if_not_exists: false,
			overwrite: false,
		})),
	)
}

// TODO(gguillemas): This test is kept in 2.0.0 for backward compatibility. Drop in 3.0.0.
#[test]
fn parse_define_token_jwks_on_scope() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE TOKEN a ON SCOPE b TYPE JWKS VALUE "http://example.com/.well-known/jwks.json" COMMENT "bar""#
	)
	.unwrap();

	// Manually compare since DefineAccessStatement for record access
	// without explicit JWT will create a random signing key during parsing.
	let Statement::Define(DefineStatement::Access(stmt)) = res else {
		panic!()
	};

	assert_eq!(stmt.name, Ident("a".to_string()));
	assert_eq!(stmt.base, Base::Db); // Scope base is ignored.
	assert_eq!(
		stmt.duration,
		// Default durations.
		AccessDuration {
			grant: Some(Duration::from_days(30).unwrap()),
			token: Some(Duration::from_hours(1).unwrap()),
			session: None,
		}
	);
	assert_eq!(stmt.comment, Some(Strand("bar".to_string())));
	assert!(!stmt.if_not_exists);
	match stmt.kind {
		AccessType::Record(ac) => {
			assert_eq!(ac.signup, None);
			assert_eq!(ac.signin, None);
			match ac.jwt.verify {
				JwtAccessVerify::Jwks(jwks) => {
					assert_eq!(jwks.url, "http://example.com/.well-known/jwks.json");
				}
				_ => panic!(),
			}
			assert_eq!(ac.jwt.issue, None);
		}
		_ => panic!(),
	}
}

// TODO(gguillemas): This test is kept in 2.0.0 for backward compatibility. Drop in 3.0.0.
#[test]
fn parse_define_scope() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE SCOPE a SESSION 1s SIGNUP true SIGNIN false COMMENT "bar""#
	)
	.unwrap();

	// Manually compare since DefineAccessStatement for record access
	// without explicit JWT will create a random signing key during parsing.
	let Statement::Define(DefineStatement::Access(stmt)) = res else {
		panic!()
	};

	assert_eq!(stmt.name, Ident("a".to_string()));
	assert_eq!(stmt.base, Base::Db);
	assert_eq!(stmt.comment, Some(Strand("bar".to_string())));
	assert_eq!(
		stmt.duration,
		AccessDuration {
			grant: Some(Duration::from_days(30).unwrap()),
			token: Some(Duration::from_hours(1).unwrap()),
			session: Some(Duration::from_secs(1)),
		}
	);
	assert!(!stmt.if_not_exists);
	match stmt.kind {
		AccessType::Record(ac) => {
			assert_eq!(ac.signup, Some(SqlValue::Bool(true)));
			assert_eq!(ac.signin, Some(SqlValue::Bool(false)));
			match ac.jwt.verify {
				JwtAccessVerify::Key(key) => {
					assert_eq!(key.alg, Algorithm::Hs512);
				}
				_ => panic!(),
			}
			match ac.jwt.issue {
				Some(iss) => {
					assert_eq!(iss.alg, Algorithm::Hs512);
				}
				_ => panic!(),
			}
		}
		_ => panic!(),
	}
}

#[test]
fn parse_define_access_jwt_key() {
	// With comment. Asymmetric verify only.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM EDDSA KEY "foo" COMMENT "bar""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::EdDSA,
						key: "foo".to_string(),
					}),
					issue: None,
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: Some(Strand("bar".to_string())),
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Asymmetric verify and issue.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM EDDSA KEY "foo" WITH ISSUER KEY "bar""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::EdDSA,
						key: "foo".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::EdDSA,
						key: "bar".to_string(),
					}),
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Asymmetric verify and issue with authenticate clause.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM EDDSA KEY "foo" WITH ISSUER KEY "bar" AUTHENTICATE true"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::EdDSA,
						key: "foo".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::EdDSA,
						key: "bar".to_string(),
					}),
				}),
				authenticate: Some(SqlValue::Bool(true)),
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Symmetric verify and implicit issue.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::Hs256,
						key: "foo".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Hs256,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Symmetric verify and explicit duration.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo" DURATION FOR TOKEN 10s"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::Hs256,
						key: "foo".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Hs256,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Symmetric verify and explicit issue matching data.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo" WITH ISSUER ALGORITHM HS256 KEY "foo""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::Hs256,
						key: "foo".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Hs256,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Symmetric verify and explicit issue non-matching data.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo" WITH ISSUER ALGORITHM HS384 KEY "bar" DURATION FOR TOKEN 10s"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of non-matching verifier and issuer: {:?}",
			res
		);
	}
	// Symmetric verify and explicit issue non-matching key.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo" WITH ISSUER KEY "bar" DURATION FOR TOKEN 10s"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of non-matching verifier and issuer: {:?}",
			res
		);
	}
	// Symmetric verify and explicit issue non-matching algorithm.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo" WITH ISSUER ALGORITHM HS384 DURATION FOR TOKEN 10s"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of non-matching verifier and issuer: {:?}",
			res
		);
	}
	// Symmetric verify and token duration is none.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT ALGORITHM HS256 KEY "foo" DURATION FOR TOKEN NONE"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of JWT access with none token duration: {:?}",
			res
		);
	}
	// With comment. Asymmetric verify only. On namespace level.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON NAMESPACE TYPE JWT ALGORITHM EDDSA KEY "foo" COMMENT "bar""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Ns,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::EdDSA,
						key: "foo".to_string(),
					}),
					issue: None,
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: Some(Strand("bar".to_string())),
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// With comment. Asymmetric verify only. On root level.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON ROOT TYPE JWT ALGORITHM EDDSA KEY "foo" COMMENT "bar""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Root,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
						alg: Algorithm::EdDSA,
						key: "foo".to_string(),
					}),
					issue: None,
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: Some(Strand("bar".to_string())),
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
}

#[test]
fn parse_define_access_jwt_jwks() {
	// With comment. Verify only.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT URL "http://example.com/.well-known/jwks.json" COMMENT "bar""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Jwks(JwtAccessVerifyJwks {
						url: "http://example.com/.well-known/jwks.json".to_string(),
					}),
					issue: None,
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: Some(Strand("bar".to_string())),
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Verify and symmetric issuer.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT URL "http://example.com/.well-known/jwks.json" WITH ISSUER ALGORITHM HS384 KEY "foo""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Jwks(JwtAccessVerifyJwks {
						url: "http://example.com/.well-known/jwks.json".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Hs384,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Verify and symmetric issuer with custom duration.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT URL "http://example.com/.well-known/jwks.json" WITH ISSUER ALGORITHM HS384 KEY "foo" DURATION FOR TOKEN 10s"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Jwks(JwtAccessVerifyJwks {
						url: "http://example.com/.well-known/jwks.json".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Hs384,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Verify and asymmetric issuer.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT URL "http://example.com/.well-known/jwks.json" WITH ISSUER ALGORITHM PS256 KEY "foo""#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Jwks(JwtAccessVerifyJwks {
						url: "http://example.com/.well-known/jwks.json".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Ps256,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				// Default durations.
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_hours(1).unwrap()),
					session: None,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// Verify and asymmetric issuer with custom duration.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DATABASE TYPE JWT URL "http://example.com/.well-known/jwks.json" WITH ISSUER ALGORITHM PS256 KEY "foo" DURATION FOR TOKEN 10s, FOR SESSION 2d"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Jwt(JwtAccess {
					verify: JwtAccessVerify::Jwks(JwtAccessVerifyJwks {
						url: "http://example.com/.well-known/jwks.json".to_string(),
					}),
					issue: Some(JwtAccessIssue {
						alg: Algorithm::Ps256,
						key: "foo".to_string(),
					}),
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_days(2).unwrap()),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
}

#[test]
fn parse_define_access_record() {
	// With comment. Nothing is explicitly defined.
	{
		let res =
			test_parse!(parse_stmt, r#"DEFINE ACCESS a ON DB TYPE RECORD COMMENT "bar""#).unwrap();

		// Manually compare since DefineAccessStatement for record access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Db);
		assert_eq!(stmt.authenticate, None);
		assert_eq!(
			stmt.duration,
			// Default durations.
			AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
		assert_eq!(stmt.comment, Some(Strand("bar".to_string())));
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Record(ac) => {
				assert_eq!(ac.signup, None);
				assert_eq!(ac.signin, None);
				match ac.jwt.verify {
					JwtAccessVerify::Key(key) => {
						assert_eq!(key.alg, Algorithm::Hs512);
					}
					_ => panic!(),
				}
				match ac.jwt.issue {
					Some(iss) => {
						assert_eq!(iss.alg, Algorithm::Hs512);
					}
					_ => panic!(),
				}
			}
			_ => panic!(),
		}
	}
	// With refresh token. Refresh token duration is set to 10 days.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD WITH REFRESH DURATION FOR GRANT 10d"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();

		// Manually compare since DefineAccessStatement for record access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Db);
		assert_eq!(stmt.authenticate, None);
		assert_eq!(
			stmt.duration,
			// Default durations.
			AccessDuration {
				grant: Some(Duration::from_days(10).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Record(ac) => {
				assert_eq!(ac.signup, None);
				assert_eq!(ac.signin, None);
				let jwt_verify_key = match ac.jwt.verify {
					JwtAccessVerify::Key(key) => {
						assert_eq!(key.alg, Algorithm::Hs512);
						key.key
					}
					_ => panic!(),
				};
				let jwt_issue_key = match ac.jwt.issue {
					Some(iss) => {
						assert_eq!(iss.alg, Algorithm::Hs512);
						iss.key
					}
					_ => panic!(),
				};
				// The JWT parameters should be the same as record authentication.
				match ac.bearer {
					Some(bearer) => {
						assert_eq!(bearer.kind, BearerAccessType::Refresh);
						assert_eq!(bearer.subject, BearerAccessSubject::Record);
						match bearer.jwt.verify {
							JwtAccessVerify::Key(key) => {
								assert_eq!(key.alg, Algorithm::Hs512);
								assert_eq!(key.key, jwt_verify_key);
							}
							_ => panic!(),
						}
						match bearer.jwt.issue {
							Some(iss) => {
								assert_eq!(iss.alg, Algorithm::Hs512);
								assert_eq!(iss.key, jwt_issue_key);
							}
							_ => panic!(),
						}
					}
					_ => panic!(),
				}
			}
			_ => panic!(),
		}
	}
	// Session duration, signing and authenticate clauses are explicitly defined.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD SIGNUP true SIGNIN false AUTHENTICATE true DURATION FOR SESSION 7d"#
		)
		.unwrap();

		// Manually compare since DefineAccessStatement for record access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Db);
		assert_eq!(stmt.authenticate, Some(SqlValue::Bool(true)));
		assert_eq!(
			stmt.duration,
			AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: Some(Duration::from_days(7).unwrap()),
			}
		);
		assert_eq!(stmt.comment, None);
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Record(ac) => {
				assert_eq!(ac.signup, Some(SqlValue::Bool(true)));
				assert_eq!(ac.signin, Some(SqlValue::Bool(false)));
				match ac.jwt.verify {
					JwtAccessVerify::Key(key) => {
						assert_eq!(key.alg, Algorithm::Hs512);
					}
					_ => panic!(),
				}
				match ac.jwt.issue {
					Some(iss) => {
						assert_eq!(iss.alg, Algorithm::Hs512);
					}
					_ => panic!(),
				}
			}
			_ => panic!(),
		}
	}
	// Verification with JWT is explicitly defined only with symmetric key.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD WITH JWT ALGORITHM HS384 KEY "foo" DURATION FOR TOKEN 10s, FOR SESSION 15m"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Record(RecordAccess {
					signup: None,
					signin: None,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Hs384,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Hs384,
							// Issuer key matches verification key by default in symmetric algorithms.
							key: "foo".to_string(),
						}),
					},
					bearer: None,
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_mins(15).unwrap()),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		);
	}
	// Verification and issuing with JWT are explicitly defined with two different keys.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD WITH JWT ALGORITHM PS512 KEY "foo" WITH ISSUER KEY "bar" DURATION FOR TOKEN 10s, FOR SESSION 15m"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Record(RecordAccess {
					signup: None,
					signin: None,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Ps512,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Ps512,
							key: "bar".to_string(),
						}),
					},
					bearer: None,
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_mins(15).unwrap()),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		);
	}
	// Verification and issuing with JWT are explicitly defined with two different keys. Refresh specified before JWT.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD WITH REFRESH WITH JWT ALGORITHM PS512 KEY "foo" WITH ISSUER KEY "bar" DURATION FOR GRANT 10d, FOR TOKEN 10s, FOR SESSION 15m"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Record(RecordAccess {
					signup: None,
					signin: None,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Ps512,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Ps512,
							key: "bar".to_string(),
						}),
					},
					bearer: Some(BearerAccess {
						kind: BearerAccessType::Refresh,
						subject: BearerAccessSubject::Record,
						jwt: JwtAccess {
							verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
								alg: Algorithm::Ps512,
								key: "foo".to_string(),
							}),
							issue: Some(JwtAccessIssue {
								alg: Algorithm::Ps512,
								key: "bar".to_string(),
							}),
						},
					}),
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(10).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_mins(15).unwrap()),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		);
	}
	// Verification and issuing with JWT are explicitly defined with two different keys. Refresh specified after JWT.
	{
		let res = test_parse_with_settings!(parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD WITH JWT ALGORITHM PS512 KEY "foo" WITH ISSUER KEY "bar" WITH REFRESH DURATION FOR GRANT 10d, FOR TOKEN 10s, FOR SESSION 15m"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		).unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Record(RecordAccess {
					signup: None,
					signin: None,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Ps512,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Ps512,
							key: "bar".to_string(),
						}),
					},
					bearer: Some(BearerAccess {
						kind: BearerAccessType::Refresh,
						subject: BearerAccessSubject::Record,
						jwt: JwtAccess {
							verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
								alg: Algorithm::Ps512,
								key: "foo".to_string(),
							}),
							issue: Some(JwtAccessIssue {
								alg: Algorithm::Ps512,
								key: "bar".to_string(),
							}),
						},
					}),
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(10).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_mins(15).unwrap()),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		);
	}
	// Verification and issuing with JWT are explicitly defined with two different keys. Token duration is explicitly defined.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE RECORD WITH JWT ALGORITHM RS256 KEY 'foo' WITH ISSUER KEY 'bar' DURATION FOR TOKEN 10s, FOR SESSION 15m"#
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Record(RecordAccess {
					signup: None,
					signin: None,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Rs256,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Rs256,
							key: "bar".to_string(),
						}),
					},
					bearer: None,
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(30).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_mins(15).unwrap()),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		);
	}
	// Verification with JWT is explicitly defined only with symmetric key. Token duration is none.
	{
		let res =
			test_parse!(parse_stmt, r#"DEFINE ACCESS a ON DB TYPE RECORD DURATION FOR TOKEN NONE"#);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of record access with none token duration: {:?}",
			res
		);
	}
	// Attempt to define record access at the root level.
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE ACCESS a ON ROOT TYPE RECORD DURATION FOR TOKEN NONE"#
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of record access at root level: {:?}",
			res
		);
	}
	// Attempt to define record access at the namespace level.
	{
		let res =
			test_parse!(parse_stmt, r#"DEFINE ACCESS a ON NS TYPE RECORD DURATION FOR TOKEN NONE"#);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of record access at namespace level: {:?}",
			res
		);
	}
}

#[test]
fn parse_define_access_bearer() {
	// For user on database.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE BEARER FOR USER COMMENT "foo""#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();

		// Manually compare since DefineAccessStatement for bearer access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Db);
		assert_eq!(stmt.authenticate, None);
		assert_eq!(
			stmt.duration,
			// Default durations.
			AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
		assert_eq!(stmt.comment, Some(Strand("foo".to_string())));
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Bearer(ac) => {
				assert_eq!(ac.subject, BearerAccessSubject::User);
			}
			_ => panic!(),
		}
	}
	// For user on namespace.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON NS TYPE BEARER FOR USER COMMENT "foo""#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();

		// Manually compare since DefineAccessStatement for bearer access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Ns);
		assert_eq!(stmt.authenticate, None);
		assert_eq!(
			stmt.duration,
			// Default durations.
			AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
		assert_eq!(stmt.comment, Some(Strand("foo".to_string())));
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Bearer(ac) => {
				assert_eq!(ac.subject, BearerAccessSubject::User);
			}
			_ => panic!(),
		}
	}
	// For user on root.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON ROOT TYPE BEARER FOR USER COMMENT "foo""#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();

		// Manually compare since DefineAccessStatement for bearer access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Root);
		assert_eq!(stmt.authenticate, None);
		assert_eq!(
			stmt.duration,
			// Default durations.
			AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
		assert_eq!(stmt.comment, Some(Strand("foo".to_string())));
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Bearer(ac) => {
				assert_eq!(ac.subject, BearerAccessSubject::User);
			}
			_ => panic!(),
		}
	}
	// For record on database.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE BEARER FOR RECORD COMMENT "foo""#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();

		// Manually compare since DefineAccessStatement for bearer access
		// without explicit JWT will create a random signing key during parsing.
		let Statement::Define(DefineStatement::Access(stmt)) = res else {
			panic!()
		};

		assert_eq!(stmt.name, Ident("a".to_string()));
		assert_eq!(stmt.base, Base::Db);
		assert_eq!(
			stmt.duration,
			// Default durations.
			AccessDuration {
				grant: Some(Duration::from_days(30).unwrap()),
				token: Some(Duration::from_hours(1).unwrap()),
				session: None,
			}
		);
		assert_eq!(stmt.comment, Some(Strand("foo".to_string())));
		assert!(!stmt.if_not_exists);
		match stmt.kind {
			AccessType::Bearer(ac) => {
				assert_eq!(ac.subject, BearerAccessSubject::Record);
			}
			_ => panic!(),
		}
	}
	// For record on namespace.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON NS TYPE BEARER FOR RECORD COMMENT "foo""#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of bearer access for record at namespace level: {:?}",
			res
		);
	}
	// For record on root.
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"DEFINE ACCESS a ON ROOT TYPE BEARER FOR RECORD COMMENT "foo""#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		);
		assert!(
			res.is_err(),
			"Unexpected successful parsing of bearer access for record at root level: {:?}",
			res
		);
	}
	// For user. Grant, session and token duration. With JWT.
	{
		let res = test_parse_with_settings!(parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE BEARER FOR USER WITH JWT ALGORITHM HS384 KEY "foo" DURATION FOR GRANT 90d, FOR TOKEN 10s, FOR SESSION 15m"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		).unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Bearer(BearerAccess {
					kind: BearerAccessType::Bearer,
					subject: BearerAccessSubject::User,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Hs384,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Hs384,
							// Issuer key matches verification key by default in symmetric algorithms.
							key: "foo".to_string(),
						}),
					},
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(90).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_secs(900)),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
	// For record. Grant, session and token duration. With JWT.
	{
		let res = test_parse_with_settings!(parse_stmt,
			r#"DEFINE ACCESS a ON DB TYPE BEARER FOR RECORD WITH JWT ALGORITHM HS384 KEY "foo" DURATION FOR GRANT 90d, FOR TOKEN 10s, FOR SESSION 15m"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		).unwrap();
		assert_eq!(
			res,
			Statement::Define(DefineStatement::Access(DefineAccessStatement {
				name: Ident("a".to_string()),
				base: Base::Db,
				kind: AccessType::Bearer(BearerAccess {
					kind: BearerAccessType::Bearer,
					subject: BearerAccessSubject::Record,
					jwt: JwtAccess {
						verify: JwtAccessVerify::Key(JwtAccessVerifyKey {
							alg: Algorithm::Hs384,
							key: "foo".to_string(),
						}),
						issue: Some(JwtAccessIssue {
							alg: Algorithm::Hs384,
							// Issuer key matches verification key by default in symmetric algorithms.
							key: "foo".to_string(),
						}),
					},
				}),
				authenticate: None,
				duration: AccessDuration {
					grant: Some(Duration::from_days(90).unwrap()),
					token: Some(Duration::from_secs(10)),
					session: Some(Duration::from_secs(900)),
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
			})),
		)
	}
}

#[test]
fn parse_define_param() {
	let res =
		test_parse!(parse_stmt, r#"DEFINE PARAM $a VALUE { a: 1, "b": 3 } PERMISSIONS WHERE null"#)
			.unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Param(DefineParamStatement {
			name: Ident("a".to_string()),
			value: SqlValue::Object(Object(
				[
					("a".to_string(), SqlValue::Number(Number::Int(1))),
					("b".to_string(), SqlValue::Number(Number::Int(3))),
				]
				.into_iter()
				.collect()
			)),
			comment: None,
			permissions: Permission::Specific(SqlValue::Null),
			if_not_exists: false,
			overwrite: false,
		}))
	);
}

#[test]
fn parse_define_table() {
	let res =
		test_parse!(parse_stmt, r#"DEFINE TABLE name DROP SCHEMAFUL CHANGEFEED 1s INCLUDE ORIGINAL PERMISSIONS FOR DELETE FULL, FOR SELECT WHERE a = 1 AS SELECT foo FROM bar GROUP BY foo"#)
			.unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Table(DefineTableStatement {
			id: None,
			name: Ident("name".to_string()),
			drop: true,
			full: true,
			view: Some(crate::sql::View {
				expr: Fields(
					vec![Field::Single {
						expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident("foo".to_owned()))])),
						alias: None,
					}],
					false
				),
				what: Tables(vec![Table("bar".to_owned())]),
				cond: None,
				group: Some(Groups(vec![Group(Idiom(vec![Part::Field(Ident("foo".to_owned()))]))])),
			}),
			permissions: Permissions {
				select: Permission::Specific(SqlValue::Expression(Box::new(
					crate::sql::Expression::Binary {
						l: SqlValue::Idiom(Idiom(vec![Part::Field(Ident("a".to_owned()))])),
						o: Operator::Equal,
						r: SqlValue::Number(Number::Int(1))
					}
				))),
				create: Permission::None,
				update: Permission::None,
				delete: Permission::Full,
			},
			changefeed: Some(ChangeFeed {
				expiry: std::time::Duration::from_secs(1),
				store_diff: true,
			}),
			comment: None,
			if_not_exists: false,
			overwrite: false,
			kind: TableType::Normal,
			cache_fields_ts: uuid::Uuid::default(),
			cache_events_ts: uuid::Uuid::default(),
			cache_tables_ts: uuid::Uuid::default(),
			cache_indexes_ts: uuid::Uuid::default(),
		}))
	);
}

#[test]
fn parse_define_event() {
	let res =
		test_parse!(parse_stmt, r#"DEFINE EVENT event ON TABLE table WHEN null THEN null,none"#)
			.unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Event(DefineEventStatement {
			name: Ident("event".to_owned()),
			what: Ident("table".to_owned()),
			when: SqlValue::Null,
			then: SqlValues(vec![SqlValue::Null, SqlValue::None]),
			comment: None,
			if_not_exists: false,
			overwrite: false,
		}))
	)
}

#[test]
fn parse_define_field() {
	// General
	{
		let res = test_parse!(
			parse_stmt,
			r#"DEFINE FIELD foo.*[*]... ON TABLE bar FLEX TYPE option<number | array<record<foo>,10>> VALUE null ASSERT true DEFAULT false PERMISSIONS FOR UPDATE NONE, FOR CREATE WHERE true"#
		).unwrap();

		assert_eq!(
			res,
			Statement::Define(DefineStatement::Field(DefineFieldStatement {
				name: Idiom(vec![
					Part::Field(Ident("foo".to_owned())),
					Part::All,
					Part::All,
					Part::Flatten,
				]),
				what: Ident("bar".to_owned()),
				flex: true,
				kind: Some(Kind::Option(Box::new(Kind::Either(vec![
					Kind::Number,
					Kind::Array(Box::new(Kind::Record(vec![Table("foo".to_owned())])), Some(10))
				])))),
				readonly: false,
				value: Some(SqlValue::Null),
				assert: Some(SqlValue::Bool(true)),
				default: Some(SqlValue::Bool(false)),
				permissions: Permissions {
					delete: Permission::Full,
					update: Permission::None,
					create: Permission::Specific(SqlValue::Bool(true)),
					select: Permission::Full,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
				reference: None,
				default_always: false,
			}))
		)
	}

	// Invalid DELETE permission
	{
		// TODO(gguillemas): Providing the DELETE permission should return a parse error in 3.0.0.
		// Currently, the DELETE permission is just ignored to maintain backward compatibility.
		let res =
			test_parse!(parse_stmt, r#"DEFINE FIELD foo ON TABLE bar PERMISSIONS FOR DELETE NONE"#)
				.unwrap();

		assert_eq!(
			res,
			Statement::Define(DefineStatement::Field(DefineFieldStatement {
				name: Idiom(vec![Part::Field(Ident("foo".to_owned())),]),
				what: Ident("bar".to_owned()),
				flex: false,
				kind: None,
				readonly: false,
				value: None,
				assert: None,
				default: None,
				permissions: Permissions {
					delete: Permission::Full,
					update: Permission::Full,
					create: Permission::Full,
					select: Permission::Full,
				},
				comment: None,
				if_not_exists: false,
				overwrite: false,
				reference: None,
				default_always: false,
			}))
		)
	}
}

#[test]
fn parse_define_index() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE INDEX index ON TABLE table FIELDS a,b[*] SEARCH ANALYZER ana BM25 (0.1,0.2)
			DOC_IDS_ORDER 1
			DOC_LENGTHS_ORDER 2
			POSTINGS_ORDER 3
			TERMS_ORDER 4
			DOC_IDS_CACHE 5
			DOC_LENGTHS_CACHE 6
			POSTINGS_CACHE 7
			TERMS_CACHE 8
			HIGHLIGHTS"#
	)
	.unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Index(DefineIndexStatement {
			name: Ident("index".to_owned()),
			what: Ident("table".to_owned()),
			cols: Idioms(vec![
				Idiom(vec![Part::Field(Ident("a".to_owned()))]),
				Idiom(vec![Part::Field(Ident("b".to_owned())), Part::All])
			]),
			index: Index::Search(SearchParams {
				az: Ident("ana".to_owned()),
				hl: true,
				sc: Scoring::Bm {
					k1: 0.1,
					b: 0.2
				},
				doc_ids_order: 1,
				doc_lengths_order: 2,
				postings_order: 3,
				terms_order: 4,
				doc_ids_cache: 5,
				doc_lengths_cache: 6,
				postings_cache: 7,
				terms_cache: 8,
			}),
			comment: None,
			if_not_exists: false,
			overwrite: false,
			concurrently: false
		}))
	);

	let res =
		test_parse!(parse_stmt, r#"DEFINE INDEX index ON TABLE table FIELDS a UNIQUE"#).unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Index(DefineIndexStatement {
			name: Ident("index".to_owned()),
			what: Ident("table".to_owned()),
			cols: Idioms(vec![Idiom(vec![Part::Field(Ident("a".to_owned()))]),]),
			index: Index::Uniq,
			comment: None,
			if_not_exists: false,
			overwrite: false,
			concurrently: false
		}))
	);

	let res =
		test_parse!(parse_stmt, r#"DEFINE INDEX index ON TABLE table FIELDS a MTREE DIMENSION 4 DISTANCE MINKOWSKI 5 CAPACITY 6 TYPE I16 DOC_IDS_ORDER 7 DOC_IDS_CACHE 8 MTREE_CACHE 9"#).unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Index(DefineIndexStatement {
			name: Ident("index".to_owned()),
			what: Ident("table".to_owned()),
			cols: Idioms(vec![Idiom(vec![Part::Field(Ident("a".to_owned()))]),]),
			index: Index::MTree(MTreeParams {
				dimension: 4,
				distance: Distance::Minkowski(Number::Int(5)),
				capacity: 6,
				doc_ids_order: 7,
				doc_ids_cache: 8,
				mtree_cache: 9,
				vector_type: VectorType::I16,
			}),
			comment: None,
			if_not_exists: false,
			overwrite: false,
			concurrently: false
		}))
	);

	let res =
		test_parse!(parse_stmt, r#"DEFINE INDEX index ON TABLE table FIELDS a HNSW DIMENSION 128 EFC 250 TYPE F32 DISTANCE MANHATTAN M 6 M0 12 LM 0.5 EXTEND_CANDIDATES KEEP_PRUNED_CONNECTIONS"#).unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Index(DefineIndexStatement {
			name: Ident("index".to_owned()),
			what: Ident("table".to_owned()),
			cols: Idioms(vec![Idiom(vec![Part::Field(Ident("a".to_owned()))]),]),
			index: Index::Hnsw(HnswParams {
				dimension: 128,
				distance: Distance::Manhattan,
				vector_type: VectorType::F32,
				m: 6,
				m0: 12,
				ef_construction: 250,
				extend_candidates: true,
				keep_pruned_connections: true,
				ml: 0.5.into(),
			}),
			comment: None,
			if_not_exists: false,
			overwrite: false,
			concurrently: false
		}))
	);
}

#[test]
fn parse_define_analyzer() {
	let res = test_parse!(
		parse_stmt,
		r#"DEFINE ANALYZER ana FILTERS ASCII, EDGENGRAM(1,2), NGRAM(3,4), LOWERCASE, SNOWBALL(NLD), UPPERCASE TOKENIZERS BLANK, CAMEL, CLASS, PUNCT FUNCTION fn::foo::bar"#
	).unwrap();

	assert_eq!(
		res,
		Statement::Define(DefineStatement::Analyzer(DefineAnalyzerStatement {
			name: Ident("ana".to_owned()),
			tokenizers: Some(vec![
				Tokenizer::Blank,
				Tokenizer::Camel,
				Tokenizer::Class,
				Tokenizer::Punct,
			]),
			filters: Some(vec![
				Filter::Ascii,
				Filter::EdgeNgram(1, 2),
				Filter::Ngram(3, 4),
				Filter::Lowercase,
				Filter::Snowball(Language::Dutch),
				Filter::Uppercase,
			]),
			comment: None,
			function: Some(Ident("foo::bar".to_string())),
			if_not_exists: false,
			overwrite: false,
		})),
	)
}

#[test]
fn parse_delete() {
	let res = test_parse!(
		parse_statement,
		"DELETE FROM ONLY |foo:32..64| WITH INDEX index,index_2 Where 2 RETURN AFTER TIMEOUT 1s PARALLEL EXPLAIN FULL"
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Delete(DeleteStatement {
			only: true,
			what: SqlValues(vec![SqlValue::Mock(crate::sql::Mock::Range(
				"foo".to_string(),
				32,
				64
			))]),
			with: Some(With::Index(vec!["index".to_owned(), "index_2".to_owned()])),
			cond: Some(Cond(SqlValue::Number(Number::Int(2)))),
			output: Some(Output::After),
			timeout: Some(Timeout(Duration(std::time::Duration::from_secs(1)))),
			parallel: true,
			explain: Some(Explain(true)),
		})
	);
}

#[test]
fn parse_delete_2() {
	let res = test_parse!(
		parse_stmt,
		r#"DELETE FROM ONLY a:b->?[$][?true] WITH INDEX index,index_2 WHERE null RETURN NULL TIMEOUT 1h PARALLEL EXPLAIN"#
	)
	.unwrap();

	assert_eq!(
		res,
		Statement::Delete(DeleteStatement {
			only: true,
			what: SqlValues(vec![SqlValue::Idiom(Idiom(vec![
				Part::Start(SqlValue::Edges(Box::new(Edges {
					dir: Dir::Out,
					from: Thing {
						tb: "a".to_owned(),
						id: Id::from("b"),
					},
					what: GraphSubjects::default(),
				}))),
				Part::Last,
				Part::Where(SqlValue::Bool(true)),
			]))]),
			with: Some(With::Index(vec!["index".to_owned(), "index_2".to_owned()])),
			cond: Some(Cond(SqlValue::Null)),
			output: Some(Output::Null),
			timeout: Some(Timeout(Duration(std::time::Duration::from_secs(60 * 60)))),
			parallel: true,
			explain: Some(Explain(false)),
		})
	)
}

#[test]
pub fn parse_for() {
	let res = test_parse!(
		parse_stmt,
		r#"FOR $foo IN (SELECT foo FROM bar) * 2 {
			BREAK
		}"#
	)
	.unwrap();

	assert_eq!(
		res,
		Statement::Foreach(ForeachStatement {
			param: Param(Ident("foo".to_owned())),
			range: SqlValue::Expression(Box::new(Expression::Binary {
				l: SqlValue::Subquery(Box::new(Subquery::Select(SelectStatement {
					expr: Fields(
						vec![Field::Single {
							expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident(
								"foo".to_owned()
							))])),
							alias: None
						}],
						false
					),
					what: SqlValues(vec![SqlValue::Table(Table("bar".to_owned()))]),
					..Default::default()
				}))),
				o: Operator::Mul,
				r: SqlValue::Number(Number::Int(2))
			})),
			block: Block(vec![Entry::Break(BreakStatement)])
		})
	)
}

#[test]
fn parse_if() {
	let res =
		test_parse!(parse_stmt, r#"IF foo THEN bar ELSE IF faz THEN baz ELSE baq END"#).unwrap();
	assert_eq!(
		res,
		Statement::Ifelse(IfelseStatement {
			exprs: vec![
				(ident_field("foo"), ident_field("bar")),
				(ident_field("faz"), ident_field("baz")),
			],
			close: Some(ident_field("baq"))
		})
	)
}

#[test]
fn parse_if_block() {
	let res =
		test_parse!(parse_stmt, r#"IF foo { bar } ELSE IF faz { baz } ELSE { baq }"#).unwrap();
	assert_eq!(
		res,
		Statement::Ifelse(IfelseStatement {
			exprs: vec![
				(
					ident_field("foo"),
					SqlValue::Block(Box::new(Block(vec![Entry::Value(ident_field("bar"))]))),
				),
				(
					ident_field("faz"),
					SqlValue::Block(Box::new(Block(vec![Entry::Value(ident_field("baz"))]))),
				)
			],
			close: Some(SqlValue::Block(Box::new(Block(vec![Entry::Value(ident_field("baq"))])))),
		})
	)
}

#[test]
fn parse_info() {
	let res = test_parse!(parse_stmt, "INFO FOR ROOT").unwrap();
	assert_eq!(res, Statement::Info(InfoStatement::Root(false)));

	let res = test_parse!(parse_stmt, "INFO FOR KV").unwrap();
	assert_eq!(res, Statement::Info(InfoStatement::Root(false)));

	let res = test_parse!(parse_stmt, "INFO FOR NAMESPACE").unwrap();
	assert_eq!(res, Statement::Info(InfoStatement::Ns(false)));

	let res = test_parse!(parse_stmt, "INFO FOR NS").unwrap();
	assert_eq!(res, Statement::Info(InfoStatement::Ns(false)));

	let res = test_parse!(parse_stmt, "INFO FOR TABLE table").unwrap();
	assert_eq!(res, Statement::Info(InfoStatement::Tb(Ident("table".to_owned()), false, None)));

	let res = test_parse!(parse_stmt, "INFO FOR USER user").unwrap();
	assert_eq!(res, Statement::Info(InfoStatement::User(Ident("user".to_owned()), None, false)));

	let res = test_parse!(parse_stmt, "INFO FOR USER user ON namespace").unwrap();
	assert_eq!(
		res,
		Statement::Info(InfoStatement::User(Ident("user".to_owned()), Some(Base::Ns), false))
	);
}

#[test]
fn parse_select() {
	let res = test_parse!(
		parse_stmt,
		r#"
SELECT bar as foo,[1,2],bar OMIT bar FROM ONLY a,1
    WITH INDEX index,index_2
    WHERE true
    SPLIT ON foo,bar
    GROUP foo,bar
    ORDER BY foo COLLATE NUMERIC ASC
    START AT { a: true }
    LIMIT BY a:b
    FETCH foo
    VERSION d"2012-04-23T18:25:43.0000511Z"
    EXPLAIN FULL
		"#
	)
	.unwrap();

	let offset = Utc.fix();
	let expected_datetime = offset
		.from_local_datetime(
			&NaiveDate::from_ymd_opt(2012, 4, 23)
				.unwrap()
				.and_hms_nano_opt(18, 25, 43, 51_100)
				.unwrap(),
		)
		.earliest()
		.unwrap()
		.with_timezone(&Utc);

	assert_eq!(
		res,
		Statement::Select(SelectStatement {
			expr: Fields(
				vec![
					Field::Single {
						expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident("bar".to_owned()))])),
						alias: Some(Idiom(vec![Part::Field(Ident("foo".to_owned()))])),
					},
					Field::Single {
						expr: SqlValue::Array(Array(vec![
							SqlValue::Number(Number::Int(1)),
							SqlValue::Number(Number::Int(2))
						])),
						alias: None,
					},
					Field::Single {
						expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident("bar".to_owned()))])),
						alias: None,
					},
				],
				false,
			),
			omit: Some(Idioms(vec![Idiom(vec![Part::Field(Ident("bar".to_owned()))])])),
			only: true,
			what: SqlValues(vec![
				SqlValue::Table(Table("a".to_owned())),
				SqlValue::Number(Number::Int(1))
			]),
			with: Some(With::Index(vec!["index".to_owned(), "index_2".to_owned()])),
			cond: Some(Cond(SqlValue::Bool(true))),
			split: Some(Splits(vec![
				Split(Idiom(vec![Part::Field(Ident("foo".to_owned()))])),
				Split(Idiom(vec![Part::Field(Ident("bar".to_owned()))])),
			])),
			group: Some(Groups(vec![
				Group(Idiom(vec![Part::Field(Ident("foo".to_owned()))])),
				Group(Idiom(vec![Part::Field(Ident("bar".to_owned()))])),
			])),
			order: Some(Ordering::Order(OrderList(vec![Order {
				value: Idiom(vec![Part::Field(Ident("foo".to_owned()))]),
				collate: true,
				numeric: true,
				direction: true,
			}]))),
			limit: Some(Limit(SqlValue::Thing(Thing {
				tb: "a".to_owned(),
				id: Id::from("b"),
			}))),
			start: Some(Start(SqlValue::Object(Object(
				[("a".to_owned(), SqlValue::Bool(true))].into_iter().collect()
			)))),
			fetch: Some(Fetchs(vec![Fetch(SqlValue::Idiom(Idiom(vec![Part::Field(Ident(
				"foo".to_owned()
			))])))])),
			version: Some(Version(SqlValue::Datetime(Datetime(expected_datetime)))),
			timeout: None,
			parallel: false,
			tempfiles: false,
			explain: Some(Explain(true)),
		}),
	);
}

#[test]
fn parse_show() {
	let res = test_parse!(parse_stmt, r#"SHOW CHANGES FOR TABLE foo SINCE 1 LIMIT 10"#).unwrap();

	assert_eq!(
		res,
		Statement::Show(ShowStatement {
			table: Some(Table("foo".to_owned())),
			since: ShowSince::Versionstamp(1),
			limit: Some(10)
		})
	);

	let offset = Utc.fix();
	let expected_datetime = offset
		.from_local_datetime(
			&NaiveDate::from_ymd_opt(2012, 4, 23)
				.unwrap()
				.and_hms_nano_opt(18, 25, 43, 51_100)
				.unwrap(),
		)
		.earliest()
		.unwrap()
		.with_timezone(&Utc);

	let res = test_parse!(
		parse_stmt,
		r#"SHOW CHANGES FOR DATABASE SINCE d"2012-04-23T18:25:43.0000511Z""#
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Show(ShowStatement {
			table: None,
			since: ShowSince::Timestamp(Datetime(expected_datetime)),
			limit: None
		})
	)
}

#[test]
fn parse_sleep() {
	let res = test_parse!(parse_stmt, r"SLEEP 1s").unwrap();

	let expect = Statement::Sleep(SleepStatement {
		duration: Duration(std::time::Duration::from_secs(1)),
	});
	assert_eq!(res, expect)
}

#[test]
fn parse_use() {
	let res = test_parse!(parse_stmt, r"USE NS foo").unwrap();
	let expect = Statement::Use(UseStatement {
		ns: Some("foo".to_owned()),
		db: None,
	});
	assert_eq!(res, expect);

	let res = test_parse!(parse_stmt, r"USE DB foo").unwrap();
	let expect = Statement::Use(UseStatement {
		ns: None,
		db: Some("foo".to_owned()),
	});
	assert_eq!(res, expect);

	let res = test_parse!(parse_stmt, r"USE NS bar DB foo").unwrap();
	let expect = Statement::Use(UseStatement {
		ns: Some("bar".to_owned()),
		db: Some("foo".to_owned()),
	});
	assert_eq!(res, expect);
}

#[test]
fn parse_use_lowercase() {
	let res = test_parse!(parse_stmt, r"use ns foo").unwrap();
	let expect = Statement::Use(UseStatement {
		ns: Some("foo".to_owned()),
		db: None,
	});
	assert_eq!(res, expect);

	let res = test_parse!(parse_stmt, r"use db foo").unwrap();
	let expect = Statement::Use(UseStatement {
		ns: None,
		db: Some("foo".to_owned()),
	});
	assert_eq!(res, expect);

	let res = test_parse!(parse_stmt, r"use ns bar db foo").unwrap();
	let expect = Statement::Use(UseStatement {
		ns: Some("bar".to_owned()),
		db: Some("foo".to_owned()),
	});
	assert_eq!(res, expect);
}

#[test]
fn parse_value_stmt() {
	let res = test_parse!(parse_stmt, r"1s").unwrap();
	let expect = Statement::Value(SqlValue::Duration(Duration(std::time::Duration::from_secs(1))));
	assert_eq!(res, expect);
}

#[test]
fn parse_throw() {
	let res = test_parse!(parse_stmt, r"THROW 1s").unwrap();

	let expect = Statement::Throw(ThrowStatement {
		error: SqlValue::Duration(Duration(std::time::Duration::from_secs(1))),
	});
	assert_eq!(res, expect)
}

#[test]
fn parse_insert() {
	let res = test_parse!(
		parse_stmt,
	r#"INSERT IGNORE INTO $foo (a,b,c) VALUES (1,2,3),(4,5,6) ON DUPLICATE KEY UPDATE a.b +?= null, c.d += none RETURN AFTER"#
	).unwrap();
	assert_eq!(
		res,
		Statement::Insert(InsertStatement {
			into: Some(SqlValue::Param(Param(Ident("foo".to_owned())))),
			data: Data::ValuesExpression(vec![
				vec![
					(
						Idiom(vec![Part::Field(Ident("a".to_owned()))]),
						SqlValue::Number(Number::Int(1)),
					),
					(
						Idiom(vec![Part::Field(Ident("b".to_owned()))]),
						SqlValue::Number(Number::Int(2)),
					),
					(
						Idiom(vec![Part::Field(Ident("c".to_owned()))]),
						SqlValue::Number(Number::Int(3)),
					),
				],
				vec![
					(
						Idiom(vec![Part::Field(Ident("a".to_owned()))]),
						SqlValue::Number(Number::Int(4)),
					),
					(
						Idiom(vec![Part::Field(Ident("b".to_owned()))]),
						SqlValue::Number(Number::Int(5)),
					),
					(
						Idiom(vec![Part::Field(Ident("c".to_owned()))]),
						SqlValue::Number(Number::Int(6)),
					),
				],
			]),
			ignore: true,
			update: Some(Data::UpdateExpression(vec![
				(
					Idiom(vec![
						Part::Field(Ident("a".to_owned())),
						Part::Field(Ident("b".to_owned())),
					]),
					Operator::Ext,
					SqlValue::Null,
				),
				(
					Idiom(vec![
						Part::Field(Ident("c".to_owned())),
						Part::Field(Ident("d".to_owned())),
					]),
					Operator::Inc,
					SqlValue::None,
				),
			])),
			output: Some(Output::After),
			version: None,
			timeout: None,
			parallel: false,
			relation: false,
		}),
	)
}

#[test]
fn parse_insert_select() {
	let res = test_parse!(parse_stmt, r#"INSERT IGNORE INTO bar (select foo from baz)"#).unwrap();
	assert_eq!(
		res,
		Statement::Insert(InsertStatement {
			into: Some(SqlValue::Table(Table("bar".to_owned()))),
			data: Data::SingleExpression(SqlValue::Subquery(Box::new(Subquery::Select(
				SelectStatement {
					expr: Fields(
						vec![Field::Single {
							expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident(
								"foo".to_string()
							))])),
							alias: None
						}],
						false
					),
					omit: None,
					only: false,
					what: SqlValues(vec![SqlValue::Table(Table("baz".to_string()))]),
					with: None,
					cond: None,
					split: None,
					group: None,
					order: None,
					limit: None,
					start: None,
					fetch: None,
					version: None,
					timeout: None,
					parallel: false,
					explain: None,
					tempfiles: false
				}
			)))),
			ignore: true,
			update: None,
			output: None,
			version: None,
			timeout: None,
			parallel: false,
			relation: false,
		}),
	)
}

#[test]
fn parse_kill() {
	let res = test_parse!(parse_stmt, r#"KILL $param"#).unwrap();
	assert_eq!(
		res,
		Statement::Kill(KillStatement {
			id: SqlValue::Param(Param(Ident("param".to_owned())))
		})
	);

	let res = test_parse!(parse_stmt, r#"KILL u"e72bee20-f49b-11ec-b939-0242ac120002" "#).unwrap();
	assert_eq!(
		res,
		Statement::Kill(KillStatement {
			id: SqlValue::Uuid(Uuid(uuid::uuid!("e72bee20-f49b-11ec-b939-0242ac120002")))
		})
	);
}

#[test]
fn parse_live() {
	let res = test_parse!(parse_stmt, r#"LIVE SELECT DIFF FROM $foo"#).unwrap();
	let Statement::Live(stmt) = res else {
		panic!()
	};
	assert_eq!(stmt.expr, Fields::default());
	assert_eq!(stmt.what, SqlValue::Param(Param(Ident("foo".to_owned()))));

	let res =
		test_parse!(parse_stmt, r#"LIVE SELECT foo FROM table WHERE true FETCH a[where foo],b"#)
			.unwrap();
	let Statement::Live(stmt) = res else {
		panic!()
	};
	assert_eq!(
		stmt.expr,
		Fields(
			vec![Field::Single {
				expr: SqlValue::Idiom(Idiom(vec![Part::Field(Ident("foo".to_owned()))])),
				alias: None,
			}],
			false,
		)
	);
	assert_eq!(stmt.what, SqlValue::Table(Table("table".to_owned())));
	assert_eq!(stmt.cond, Some(Cond(SqlValue::Bool(true))));
	assert_eq!(
		stmt.fetch,
		Some(Fetchs(vec![
			Fetch(SqlValue::Idiom(Idiom(vec![
				Part::Field(Ident("a".to_owned())),
				Part::Where(SqlValue::Idiom(Idiom(vec![Part::Field(Ident("foo".to_owned()))]))),
			]))),
			Fetch(SqlValue::Idiom(Idiom(vec![Part::Field(Ident("b".to_owned()))]))),
		])),
	)
}

#[test]
fn parse_option() {
	let res = test_parse!(parse_stmt, r#"OPTION value = true"#).unwrap();
	assert_eq!(
		res,
		Statement::Option(OptionStatement {
			name: Ident("value".to_owned()),
			what: true
		})
	)
}

#[test]
fn parse_return() {
	let res = test_parse!(parse_stmt, r#"RETURN RETRUN FETCH RETURN"#).unwrap();
	assert_eq!(
		res,
		Statement::Output(OutputStatement {
			what: ident_field("RETRUN"),
			fetch: Some(Fetchs(vec![Fetch(ident_field("RETURN"))]))
		}),
	)
}

#[test]
fn parse_relate() {
	let res = test_parse!(
		parse_stmt,
		r#"RELATE ONLY [1,2]->a:b->(CREATE foo) UNIQUE SET a += 1 RETURN NONE PARALLEL"#
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Relate(RelateStatement {
			only: true,
			kind: SqlValue::Thing(Thing {
				tb: "a".to_owned(),
				id: Id::from("b"),
			}),
			from: SqlValue::Array(Array(vec![
				SqlValue::Number(Number::Int(1)),
				SqlValue::Number(Number::Int(2)),
			])),
			with: SqlValue::Subquery(Box::new(Subquery::Create(CreateStatement {
				only: false,
				what: SqlValues(vec![SqlValue::Table(Table("foo".to_owned()))]),
				data: None,
				output: None,
				timeout: None,
				parallel: false,
				version: None,
			}))),
			uniq: true,
			data: Some(Data::SetExpression(vec![(
				Idiom(vec![Part::Field(Ident("a".to_owned()))]),
				Operator::Inc,
				SqlValue::Number(Number::Int(1))
			)])),
			output: Some(Output::None),
			timeout: None,
			parallel: true,
		}),
	)
}

#[test]
fn parse_remove() {
	let res = test_parse!(parse_stmt, r#"REMOVE NAMESPACE ns"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Namespace(RemoveNamespaceStatement {
			name: Ident("ns".to_owned()),
			if_exists: false,
			expunge: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE DB database"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Database(RemoveDatabaseStatement {
			name: Ident("database".to_owned()),
			if_exists: false,
			expunge: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE FUNCTION fn::foo::bar"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Function(RemoveFunctionStatement {
			name: Ident("foo::bar".to_owned()),
			if_exists: false,
		}))
	);
	let res = test_parse!(parse_stmt, r#"REMOVE FUNCTION fn::foo::bar();"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Function(RemoveFunctionStatement {
			name: Ident("foo::bar".to_owned()),
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE ACCESS foo ON DATABASE"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Access(RemoveAccessStatement {
			name: Ident("foo".to_owned()),
			base: Base::Db,
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE PARAM $foo"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Param(RemoveParamStatement {
			name: Ident("foo".to_owned()),
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE TABLE foo"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Table(RemoveTableStatement {
			name: Ident("foo".to_owned()),
			if_exists: false,
			expunge: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE EVENT foo ON TABLE bar"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Event(RemoveEventStatement {
			name: Ident("foo".to_owned()),
			what: Ident("bar".to_owned()),
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE FIELD foo.bar[10] ON bar"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Field(RemoveFieldStatement {
			name: Idiom(vec![
				Part::Field(Ident("foo".to_owned())),
				Part::Field(Ident("bar".to_owned())),
				Part::Index(Number::Int(10))
			]),
			what: Ident("bar".to_owned()),
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE INDEX foo ON bar"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Index(RemoveIndexStatement {
			name: Ident("foo".to_owned()),
			what: Ident("bar".to_owned()),
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE ANALYZER foo"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::Analyzer(RemoveAnalyzerStatement {
			name: Ident("foo".to_owned()),
			if_exists: false,
		}))
	);

	let res = test_parse!(parse_stmt, r#"REMOVE user foo on database"#).unwrap();
	assert_eq!(
		res,
		Statement::Remove(RemoveStatement::User(RemoveUserStatement {
			name: Ident("foo".to_owned()),
			base: Base::Db,
			if_exists: false,
		}))
	);
}

#[test]
fn parse_update() {
	let res = test_parse!(
		parse_stmt,
		r#"UPDATE ONLY <future> { "text" }, a->b WITH INDEX index,index_2 UNSET foo... , a->b, c[*] WHERE true RETURN DIFF TIMEOUT 1s PARALLEL EXPLAIN FULL"#
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Update(UpdateStatement {
			only: true,
			what: SqlValues(vec![
				SqlValue::Future(Box::new(Future(Block(vec![Entry::Value(SqlValue::Strand(
					Strand("text".to_string())
				))])))),
				SqlValue::Idiom(Idiom(vec![
					Part::Field(Ident("a".to_string())),
					Part::Graph(Graph {
						dir: Dir::Out,
						what: GraphSubjects(vec![GraphSubject::Table(Table("b".to_string()))]),
						..Default::default()
					})
				]))
			]),
			with: Some(With::Index(vec!["index".to_owned(), "index_2".to_owned()])),
			cond: Some(Cond(SqlValue::Bool(true))),
			data: Some(Data::UnsetExpression(vec![
				Idiom(vec![Part::Field(Ident("foo".to_string())), Part::Flatten]),
				Idiom(vec![
					Part::Field(Ident("a".to_string())),
					Part::Graph(Graph {
						dir: Dir::Out,
						what: GraphSubjects(vec![GraphSubject::Table(Table("b".to_string()))]),
						..Default::default()
					})
				]),
				Idiom(vec![Part::Field(Ident("c".to_string())), Part::All])
			])),
			output: Some(Output::Diff),
			timeout: Some(Timeout(Duration(std::time::Duration::from_secs(1)))),
			parallel: true,
			explain: Some(Explain(true))
		})
	);
}

#[test]
fn parse_upsert() {
	let res = test_parse!(
		parse_stmt,
		r#"UPSERT ONLY <future> { "text" }, a->b WITH INDEX index,index_2 UNSET foo... , a->b, c[*] WHERE true RETURN DIFF TIMEOUT 1s PARALLEL EXPLAIN"#
	)
	.unwrap();
	assert_eq!(
		res,
		Statement::Upsert(UpsertStatement {
			only: true,
			what: SqlValues(vec![
				SqlValue::Future(Box::new(Future(Block(vec![Entry::Value(SqlValue::Strand(
					Strand("text".to_string())
				))])))),
				SqlValue::Idiom(Idiom(vec![
					Part::Field(Ident("a".to_string())),
					Part::Graph(Graph {
						dir: Dir::Out,
						what: GraphSubjects(vec![GraphSubject::Table(Table("b".to_string()))]),
						..Default::default()
					})
				]))
			]),
			with: Some(With::Index(vec!["index".to_owned(), "index_2".to_owned()])),
			cond: Some(Cond(SqlValue::Bool(true))),
			data: Some(Data::UnsetExpression(vec![
				Idiom(vec![Part::Field(Ident("foo".to_string())), Part::Flatten]),
				Idiom(vec![
					Part::Field(Ident("a".to_string())),
					Part::Graph(Graph {
						dir: Dir::Out,
						what: GraphSubjects(vec![GraphSubject::Table(Table("b".to_string()))]),
						..Default::default()
					})
				]),
				Idiom(vec![Part::Field(Ident("c".to_string())), Part::All])
			])),
			output: Some(Output::Diff),
			timeout: Some(Timeout(Duration(std::time::Duration::from_secs(1)))),
			parallel: true,
			explain: Some(Explain(false))
		})
	);
}

#[test]
fn parse_access_grant() {
	// User
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON NAMESPACE GRANT FOR USER b"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Grant(AccessStatementGrant {
				ac: Ident("a".to_string()),
				base: Some(Base::Ns),
				subject: access::Subject::User(Ident("b".to_string())),
			}))
		);
	}
	// Record
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON NAMESPACE GRANT FOR RECORD b:c"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Grant(AccessStatementGrant {
				ac: Ident("a".to_string()),
				base: Some(Base::Ns),
				subject: access::Subject::Record(Thing {
					tb: "b".to_owned(),
					id: Id::from("c"),
				}),
			}))
		);
	}
}

#[test]
fn parse_access_show() {
	// All
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE SHOW ALL"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Show(AccessStatementShow {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				gr: None,
				cond: None,
			}))
		);
	}
	// Grant
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE SHOW GRANT b"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Show(AccessStatementShow {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				gr: Some(Ident("b".to_string())),
				cond: None,
			}))
		);
	}
	// Condition
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE SHOW WHERE true"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Show(AccessStatementShow {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				gr: None,
				cond: Some(Cond(SqlValue::Bool(true))),
			}))
		);
	}
}

#[test]
fn parse_access_revoke() {
	// All
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE REVOKE ALL"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Revoke(AccessStatementRevoke {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				gr: None,
				cond: None,
			}))
		);
	}
	// Grant
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE REVOKE GRANT b"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Revoke(AccessStatementRevoke {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				gr: Some(Ident("b".to_string())),
				cond: None,
			}))
		);
	}
	// Condition
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE REVOKE WHERE true"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Revoke(AccessStatementRevoke {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				gr: None,
				cond: Some(Cond(SqlValue::Bool(true))),
			}))
		);
	}
}

#[test]
fn parse_access_purge() {
	// All
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE PURGE EXPIRED, REVOKED"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Purge(AccessStatementPurge {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				expired: true,
				revoked: true,
				grace: Duration::from_millis(0),
			}))
		);
	}
	// Expired
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE PURGE EXPIRED"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Purge(AccessStatementPurge {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				expired: true,
				revoked: false,
				grace: Duration::from_millis(0),
			}))
		);
	}
	// Revoked
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE PURGE REVOKED"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Purge(AccessStatementPurge {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				expired: false,
				revoked: true,
				grace: Duration::from_millis(0),
			}))
		);
	}
	// Expired for 90 days
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE PURGE EXPIRED FOR 90d"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Purge(AccessStatementPurge {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				expired: true,
				revoked: false,
				grace: Duration::from_days(90).unwrap(),
			}))
		);
	}
	// Revoked for 90 days
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE PURGE REVOKED FOR 90d"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Purge(AccessStatementPurge {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				expired: false,
				revoked: true,
				grace: Duration::from_days(90).unwrap(),
			}))
		);
	}
	// Invalid for 90 days
	{
		let res = test_parse_with_settings!(
			parse_stmt,
			r#"ACCESS a ON DATABASE PURGE REVOKED, EXPIRED FOR 90d"#,
			ParserSettings {
				bearer_access_enabled: true,
				..Default::default()
			}
		)
		.unwrap();
		assert_eq!(
			res,
			Statement::Access(AccessStatement::Purge(AccessStatementPurge {
				ac: Ident("a".to_string()),
				base: Some(Base::Db),
				expired: true,
				revoked: true,
				grace: Duration::from_days(90).unwrap(),
			}))
		);
	}
}

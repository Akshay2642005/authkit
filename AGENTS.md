AuthKit

A better-auth–inspired authentication library for Rust
Plug-and-play. Framework-agnostic. Opinionated, but extensible.

Overview

AuthKit is a Rust authentication library designed to feel like better-auth, but for the Rust ecosystem.

It provides:

A single Auth entry point

Opinionated defaults (secure by default)

Zero framework lock-in

Database-backed authentication using SQLx (Postgres / SQLite)

The same API across HTTP servers, CLIs, background workers, and proxies

AuthKit is not a framework, middleware, or ORM.
It is a self-contained authentication engine that you embed into your application.

Design Goals

AuthKit is built around the following non-negotiable principles:

1. Single Entry Point

Users interact with one object only: Auth.

let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;


No repositories.
No generics.
No leaked internals.

2. Framework-Agnostic by Design

AuthKit:

Does not depend on Axum, Actix, Rocket, Hyper, or Tower

Does not assume HTTP

Works equally well in:

CLI tools

REST APIs

gRPC services

Proxies (Pingora)

Background workers

Framework adapters live outside the core.

3. Opinionated Defaults, Explicit Overrides

AuthKit ships with:

Argon2id password hashing

Database-backed sessions

Secure token generation

Sensible expiry defaults

Users can override behavior explicitly, but never accidentally weaken security.

4. No Leaky Abstractions

AuthKit hides:

SQLx

Database schemas

Crypto implementations

Token formats

Users never interact with:

Traits

Repositories

Lifetimes

Generic parameters

5. Same API Everywhere
auth.register(Register { ... }).await?;
auth.login(Login { ... }).await?;
auth.verify(token).await?;
auth.logout(token).await?;


These calls behave identically whether invoked from:

an HTTP handler

a CLI command

a test

a background task

Non-Goals

AuthKit intentionally does not attempt to:

Be an OAuth provider (may integrate later)

Replace application authorization logic

Act as a user management UI

Tie itself to any web framework

Example Usage
Create an Auth Instance
use authkit::prelude::*;

let auth = Auth::builder()
    .database(Database::sqlite("auth.db").await?)
    .build()?;

Register a User
auth.register(Register {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;

Login
let session = auth.login(Login {
    email: "user@example.com".into(),
    password: "secure-password".into(),
}).await?;

Verify a Session
let user = auth.verify(&session.token).await?;

Logout
auth.logout(&session.token).await?;

Architecture
Auth
 └── AuthInner (Arc)
     ├── Database (trait object)
     ├── PasswordStrategy
     └── SessionStrategy


Key characteristics:

Auth is cheap to clone

Internals are hidden

Components are swappable

No global state

Database Support

Currently supported:

SQLite

PostgreSQL

Backed by SQLx, but SQLx is not exposed.

AuthKit manages:

Schema

Migrations

Versioning

auth.migrate().await?;

Security Defaults
Feature	Default
Password hashing	Argon2id
Timing-safe compares	Enabled
Session expiration	Enabled
Token entropy	High
Password reuse	Prevented
Weak passwords	Rejected

Security-sensitive behavior requires explicit opt-out.

Feature Flags
[features]
default = ["sqlite", "argon2"]

sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]
argon2 = []
jwt = []

Adapters

Adapters translate framework-specific concepts into AuthKit calls.

Planned adapters:

Axum

Actix

Rocket

Hyper

Pingora

CLI helpers

Adapters contain no authentication logic.

Project Status

Current phase: Foundation

Implemented / In Progress:

Core Auth API

Builder pattern

SQLite backend

Password hashing

Database sessions

Planned:

PostgreSQL backend

Axum adapter

JWT sessions

Refresh tokens

Audit logging

Contribution Guidelines (Agents)

If you are contributing to this project:

You MUST:

Preserve the single-entry-point design

Avoid exposing generics or traits publicly

Keep framework dependencies out of core

Prefer composition over configuration

Default to secure behavior

You MUST NOT:

Add framework-specific logic to core

Leak SQLx types into the public API

Introduce global state

Require users to wire repositories manually

If a change makes the API feel less like better-auth, it is probably wrong.

License

MIT / Apache-2.0 (TBD)

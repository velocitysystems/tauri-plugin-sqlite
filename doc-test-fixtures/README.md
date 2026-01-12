# Doc Test Fixtures

This directory contains fixtures required for documentation examples to
compile.

## migrations/

Dummy migration files used by `sqlx::migrate!()` in doc examples. These
allow the examples to use \`\`\`no_run instead of \`\`\`ignore, ensuring they
compile and stay up-to-date with API changes.

The migrations are minimal and never executed - they exist solely for
compile-time validation.

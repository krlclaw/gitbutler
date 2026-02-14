# Case 37: Check Unread Updates (Parent Dir Overlap)

Coordination usefulness: when an agent posts an update that mentions a directory (for example `src/`),
another agent checking a file under that directory (for example `src/app.txt`) should see that message
as an unread relevant update, and the unread cursor should advance so the update is not repeated on
subsequent checks.


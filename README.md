I'm not sure what this project will be yet. Right now, it's a combination
between following [Zero to Production in Rust](https://www.zero2prod.com/) (which is a great book that you should look into if you're interested in Rust!) and setting up a side project of my own.

# Technical Choices That Differ from Zero2Prod

## Database

In Zero2Prod, Luca uses SQLx for almost all of the database connectivity: connection pooling, executing queries, and running migrations. Additionally, he uses Postgres. I'll be taking a different approach: I'll use `deadpool` for connection pooling, `rusqlite` (through `deadpool_sqlite`) for executing queries, and `refinery` for running migrations. And, as you may have guessed, SQLite for the underlying database. Under normal circumstances, I'd probably chooses the conventional approach of using a client/server RDBMS like Postgres, but I'm using this setup because:

- I want to try out [Litestream](https://litestream.io/) and see how it works. It seems promising, and what better place to try it out than a silly little side project :)
- SQLite is _much_ simpler to set up and work with and has virtually no spin up time. This makes integration tests run against it super fast, and I get the same confidence in my queries because I'm still running against the real-deal database I'm going to use.
- I couldn't set up `testcontainers` like I wanted to. I gave it a shot using Postgres at first (which you can see in earlier commits), but had issues running it within an async context. There is an experimental HTTP client that is async, but I couldn't get that to work either... If you have any ideas on how to use `testcontainers` in async the right way, I'm all ears :)
- I felt like SQLx was overkill for a project like this. While I love the idea of type-checking your SQL queries -- either through ORMs like Diesel or through the macros SQLx exposes -- I didn't feel like onboarding the extra complexity was worth it for this little toy project. Are they great tools? Yes! Should you use them if they make sense for you or you like them? Also yes! I just wanted to keep it as simple as possible

## Error Handling

I'm going to be honest, I didn't get to this point in the book (Chapter 8) before implementing what I did. Maybe I'll change it? Luca uses both `anyhow` and `thiserror` for different use cases, while I'm currently using `snafu`. I like `snafu` because:

- The `Whatever` error lets me just... not care about the error in certain scenarios. This isn't ideal in most cases, but it works great for the start up process. I don't care about what the error is, since it'll just terminate the program anyways; but, I don't have to panic.
- It makes it easy to map errors from dependencies to my custom errors using the `context` function added by the library.
- It's easy to map these errors to HTTP responses by leveraging Actix Web traits. It's probably possible to do the same thing with `thiserror` or `anyhow`, but I'm content with how `snafu` does it.

You may disagree with these points, or your mileage may vary when using them. For now, my decision makes it easy to do what I want to do, so I will stick with it.

# Plans

- Use fly.io for deployment. Not sure how this will work with Litestream, but we'll see!
- Make a multiplayer game using web sockets. Probably something turn based like a card game.
    - Might be derivative with it and just... use an existing card game (hell, I might even copy my previous project I did with a friend)

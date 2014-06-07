# Library design

This document discusses the Rust date time library's design decisions and provides a rationale.

## Philosophy

The library design philosophy is inspired by Noda Time's [design philosophy and conventions][nodatime-philosophy].

[nodatime-philosophy]: http://nodatime.org/1.1.x/userguide/design.html

* We will handle 99% of the use cases well, the other 1% can write their own libraries.  Supporting the 1% use cases creates unwanted complexity for the majority of users, and it is difficult predict what the 1% use cases even are.

* Important concepts belong in the type system.  For example, the difference between local and universal time, even though both can be written in the same way.

## Leap seconds

No support for leap seconds is planned.

### Rationale

These are too difficult to get right, require keeping an up to date database just to do basic interval arithmetic (or results might differ!), slow things down in the common use cases, and are ignored by almost every existing date and time library or API on the planet.  NTP is an exception, it handles leap seconds, the task of writing an NTP client certainly falls in the 1% of use cases we don't want to handle.

## Instants and durations

Instants and durations will be distinct types which wrap counds of 100-nanosecond ticks stored in an `i64` field.  The epoch for instants is 2000-01-01T00:00:00 UTC.

### Rationale

The *instant* and *duration* are the most fundamental units of absolute time.  They will have the same underlying format, but since they support different conceptual operations, they will have different types.  Put a different way, the timeline of instants is an *affine space* ([Wikipedia][affine-space]) but the space of durations is a *vector space* ([Wikipedia][vector-space]).

[affine-space]: http://en.wikipedia.org/wiki/Affine_space
[vector-space]: http://en.wikipedia.org/wiki/Vector_space

By choosing the data types and units for instants and durations we choose both the resolution and range of these types.  Possible choices:

* Nanoseconds in `i64`.  Pros: small, easy arithmetic.  Cons: range ±292 years.

* 100 nanoseconds in `i64`.  Pros: small, easy arithmetic, range ±29,200 years.  Cons: not enough precision for some profiling tasks, less precision than some APIs.

* Microseconds in `i64`.  Pros: small, easy arithmetic, range ±292,000 years.  Cons: not enough precision for some profiling tasks, less precision than some APIs.

* Higher-precision values.  This includes structures which store a nanosecond precision field in addition to other fields with lower precision, in order to increase the range.  Pros: high precision and range.  Cons: storage is 12 bytes or more, arithmetic is more complicated.

* Seconds in `f64`.  Pros: small, easy arithmetic, microsecond precision gives range of ±2,280 years.  Cons: variable precision may encourage use of API for sensitive performance measurements which only have nanosecond precision within 2 years of epoch.  Arithmetic is not associative and may give surprising results.

However, it may be wasteful to design the general case to support all profiling tasks.  Profiling tasks will likely use timers running at a rate only known at runtime, so for profiling tasks it makes the most sense to do arithmetic using the actual timer resolution (such as clock cycles) before converting to durations.  Worse yet, performance clocks already have better than nanosecond precision, and the precision will only increase in the future.

High precision profiling tasks also have no use for UTC.  Since profiling events happen so quickly, only local clocks can be used for measuring them.

The choice of 100-nanosecond ticks in an `i64` field keeps arithmetic easy and fast, uses minimal storage space, represents all of recorded human history, and has enough precision for all but certain profiling tasks.  This is the same engineering trade-off made by the .NET framework and Noda Time, and only one order of magnitude greater than Java and Joda Time's precision.  It is assumed that high performance profiling will be done through a separate API.

With the available range, the choice of epoch is somewhat arbitrary.  The epoch is intended to be an implementation detail.  Choosing the beginning of a year which is divisible by 400 slightly simplifies certain calendrical calculations when using the ISO 8601 calendar, which is why the year 2000 was chosen.  This epoch is also used by PostgreSQL.

Some may wonder why 1970 was not chosen, since it is such a common choice.  Choosing a common epoch only provides a slight benefit to interoperability, since converting between different formats already requires changing the precision.  Different systems already use different epochs, and the type system will help prevent users from accidentally interpreting the value stored in an instant as if it were relative to a different epoch.

# Library design

This document discusses the Rust date time library's design decisions and provides a rationale.

## Philosophy

The library design philosophy is inspired by Noda Time's [design philosophy and conventions][nodatime-philosophy].

[nodatime-philosophy]: http://nodatime.org/1.1.x/userguide/design.html

* We will handle 99% of the use cases well, the other 1% can write their own libraries.  Supporting the 1% use cases creates unwanted complexity for the majority of users, and it is difficult predict what the 1% use cases even are.

* Important concepts belong in the type system.  For example, the difference between local and universal time, even though both can be written in the same way.

## Time scales and leap seconds

The default Rust time scale divides each day into 86,400 seconds, which may or may not be equal to the SI unit.  At any given moment, the number of seconds since midnight on the Rust time scale must nominally differ by no more than 1 from the number of seconds since the same midnight according to the civil calendar.

### Rationale

The dirty reality is that some people want SI seconds, some people want to match the civil calendar, some people want to measure the rotation of the Earth, the APIs provided by common operating systems don't support any of these things, and converting between the different time scales cannot be done in software without delivering updated conversion tables on a regular basis.

Our chief goal is software reliability and interoperability with common systems, which means using an underspecified time scale without leap seconds but which tracks UTC relatively closely.  It is understandable that some software developers want leap seconds abolished.

As an example use case, consider a program that synchronizes a local file with one fetched using HTTP.  The program reads a timestamp stored on disk, submits a conditional request to the server using the If-Modified-Since header, and then changes the timestamp on disk to match the Last-Modified header sent by the server.  Any library which tries to do something "smart" to translate between the HTTP header format (which is nominally UTC) and the numeric timestamp on disk risks the catastrophe of disagreeing with other programs which interact with the same file or server.  Therefore, the conversion between numeric timestamps and calendrical dates and times must do the simplest, dumbest thing, which agrees with all the other systems in practice.

So, what happens to precise measurements based on UTC or TAI?  These can be provided by an additional API, if support is available.  It is becoming more common to encounter hardware with GPS receivers, for example, which provide access not only to TAI but to leap second information.

## Instants and durations

Instants and durations are distinct types which wrap counds of 100-nanosecond ticks stored in an `i64` field.  The epoch for instants is 2000-01-01T00:00:00 UTC.

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

## Operator overloading

The Duration type supports addition and subtraction, and can be post-multiplied by 64-bit integers.

The Instant type, while superficially similar, only supports post addition with Durations.

### Rationale

While it may be tempting to support computations with instants such as using T2 - T1 to compute the duration between the two instants, instants do not have group structure and therefore should not use group notation.  For example, while T1 + (T2 - T3) makes sense, reparenthesizing it as (T1 + T2) - T3 results in garbage.  This violates the expected behavior of the + and - operators (rounding errors in floating point arithmetic notwithstanding).

## Serialization formats

Durations are serialized in the ISO 8601 duration format `PTnnn.nnnS`, where `nnn.nnn` is the decimal length of the duration, in seconds, whith any precision.  As an extension to ISO 8601, the number may be negative, giving the format `PT-nnn.nnnS`.

Instants are serialized using the ISO 8601 format for date and time, using `T` to separate date and time, and using the `Z` suffix to indicate UTC.  For example, `2014-06-10T11:12:13.456Z`.  Leap seconds are not accounted for.

### Rationale

With the exception of the sign, this is the format specified by ISO 8601 for periods.  Since a duration actually measures seconds, it makes no sense to report a duration using hours and minutes, even if the representation would be more compact.  Leap seconds can cause the true duration of larger units to vary anyway.  The sign matches the format used by JSR-310, although JSR-310 is more particular about the permitted precision.

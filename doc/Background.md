# Background information

This document discusses background information about date and time systems.

## Standards

* ISO 80000-3:2006 ([Wikipedia][iso-8000-3-2006]), which supersedes ISO 31-1 and ISO 31-2, defines quantities and units for space and time.  It defines the actual length of a second.

* ISO 8601 ([Wikipedia][iso-8601]) defines a calendar and the representation for dates and times using that calendar.

* [Recommendation ITU-R TF.460-5][itu-r-tf-460-5] describes time scales UT0, UT1, UT2, TAI, and UTC.

* The Gregorian Calendar "is internationally the most widely used civil calendar" ([Wikipedia][gregorian-calendar]).  Note that this differs slightly from ISO 8601 for dates in the far past.

* The Julian Day ([Wikipedia][julian-day]) is the measurement of time by counting the number of days since a certain day.  The system is used by astronomers.

* Unix time, or POSIX time ([Wikipedia][unix-time]), describes instants in time relative to UTC.  POSIX time is, surprisingly, non-monotonic.

* The Network Time Protocol ([Wikipedia][ntp]) is a system for synchronizing clocks over the network.

* [The IANA Time Zone Database][iana-time-zones] tracks the historical differences between UTC and local time for different time zones.

[iso-8000-3-2006]: http://en.wikipedia.org/wiki/ISO_80000-3
[iso-8601]: http://en.wikipedia.org/wiki/ISO_8601
[itu-r-tf-460-5]: http://www.itu.int/dms_pubrec/itu-r/rec/tf/R-REC-TF.460-5-199710-S!!PDF-E.pdf
[gregorian-calendar]: http://en.wikipedia.org/wiki/Gregorian_calendar
[unix-time]: http://en.wikipedia.org/wiki/Unix_Time
[ntp]: http://en.wikipedia.org/wiki/Network_Time_Protocol
[iana-time-zones]: http://www.iana.org/time-zones
[julian-day]: http://en.wikipedia.org/wiki/Julian_day

## Existing date and time libraries or discussions

* [Joda-Time: Java date and time API][jodatime]

* [Noda Time: A better date and time API for .NET][nodatime] (also see the Noda Time [design philosophy and conventions][nodatime-philosophy])

* [JSR-310 Multi calendar system][jsr-310-multi]: A discussion of multi-calendar API design.

* [Proposal to Add Date-Time to the C++ Standard Library][cpp-proposal]

[jodatime]: http://www.joda.org/joda-time/
[nodatime]: http://nodatime.org/
[nodatime-philosophy]: http://nodatime.org/1.2.x/userguide/design.html
[jsr-310-multi]: https://github.com/ThreeTen/threeten/wiki/Multi-calendar-system
[cpp-proposal]: http://www.crystalclearsoftware.com/libraries/date_time/proposal_v75/date_time_standard.html

## Resolution

What resolution do various clocks use?

* Seconds.  This is the coarsest common granularity for instants. It is precise enough for displaying timestamps to users, but not precise enough for network applications.  The `time()` function on POSIX returns a timestamp using second precision.

* Milliseconds.  This is precise enough for most network applications but not precise enough for measuring performance.  This is the rough limit of accuracy for instants on commercial hardware.  Windows provides a count of milliseconds since boot using the `GetTickCount()` function.  Joda Time uses millisecond precision.

* Microseconds.  This is precise enough for nearly all network applications (light travels only 300 feet) and sometimes precise enough for measuring performance.  POSIX systems provide this resolution through the `timeval` structure and `gettimeofday()`. Comparing microseconds timestamps generated on independent systems is generally meaningless due to clock drift on typical computer hardware.  The .NET framework and Noda Time use 0.1 microsecond precision.

* Nanoseconds.  This is precise enough for measuring performance, since it approaches cycle granularity on modern systems.  POSIX systems provide this resolution through the `timespec` structure and `clock_gettime()`.  Comparing nanosecond timestamps generated on different physical processors on the same system can reveal problems with monotonicity.

#### References:

* [The Open Group Base Specifications Issue 6: Get the date and time][posix-gettimeofday]: "The `gettimeofday()` function shall obtain the current time, expressed as seconds and microseconds since the Epoch..."
* [Windows Dev Center - Desktop: GetTickCount function][msdn-gettickcount]: "The return value is the number of milliseconds that have elapsed since the system was started."
* [.NET Framework 4.5: DateTimeOffset.Ticks Property][msdn-datetimeoffset-ticks]:  "The value of the `Ticks` property represents the number of 100-nanosecond intervals that have elapsed since 12:00:00 midnight on January 1, 0001..."
* [Joda-Time: Instant][joda-instant]: "An Instant is defined as *an instant in the datetime continuum specified as a number of milliseconds from 1970-01-01T00:00Z.*"
* [The NTP FAQ and HOWTO][ntp-faq]

David Dalton wrote about clock offset when using NTP, as cited in the NTP FAQ and HOWTO:

> The true answer is: It All Depends.....

> Mostly, it depends on your networking. Sure, you can get your machines within a few milliseconds of each other if they are connected to each other with normal 10-Base-T Ethernet connections and not too many routers hops in between. If all the machines are on the same quiet subnet, NTP can easily keep them within one millisecond all the time. But what happens if your network get congested? What happens if you have a broadcast storm (say 1,000 broadcast packets per second) that causes your CPU to go over 100% load average just examining and discarding the broadcast packets? What happens if one of your routers loses its mind? Your local system time could drift well outside the "few milliseconds" window in situations like these.

[posix-gettimeofday]: http://pubs.opengroup.org/onlinepubs/009695399/functions/gettimeofday.html
[msdn-gettickcount]: http://msdn.microsoft.com/en-us/library/windows/desktop/ms724408(v=vs.85).aspx
[msdn-datetimeoffset-ticks]: http://msdn.microsoft.com/en-us/library/system.datetimeoffset.ticks.aspx
[joda-instant]: http://www.joda.org/joda-time/key_instant.html
[ntp-faq]: http://www.ntp.org/ntpfaq/NTP-s-algo.htm

## Range

32-bit signed Unix time creates problems with wrapping in the year 2038.  64-bit Unix time and other modern systems do not have this issue.

## Conceptual data types

These are the conceptual data types common in other date-time libraries.  These do not need to correspond directly to actual Rust data types.

* An **Instant** is an absolute moment in time, measured relative to UTC.  Instants may be stored on disk or transmitted across the network and compared against instants generated on other systems. Events from multiple systems will not always have monotonic timestamps due to clock drift.  An instant cannot be converted to a local date time without knowledge of the time zone.

* A **Duration** is an absolute amount of time.  The difference between two intervals is a duration.

* A **Zoned Date Time** is an absolute moment in time with an associated time zone.  A zoned date time can be converted to an instant by discarding the time zone.  This can also be converted to a local date time.

* An **Offset Date Time** is an absolute moment in time with an offset from UTC.  An offset date time can be converted to an instant by discarding the offset.  This can also be converted to a local date time.

* A **Local Date Time** is a moment in time according to the calendar system in use.  For example, 2014-01-04, 11:30AM.  This cannot in general be converted to an instant, since the same local time will correspond to different instants in different time zones, and will sometimes correspond to zero or multiple instants in a particular time zone.

* A **Local Date** is a date according to the calendar system.

* A **Local Time** is the time of day.

* A **Period** is a relative amount of time in calendrical terms, such as hours, weeks, or months.  The absolute length of a period (as a duration) will generally differ.

### Period arithmetic

Period arithmetic is not straightforward, because adding certain periods to dates will result in invalid dates.

For example, "one month after May 31" cannot be interpreted unambiguously, since "June 31" is an invalid date.  June 30 and July 1 are reasonable interpretations, or an implementation may signal an error.

#### References

* [Noda Time: Date and time arithmetic][nodatime-arithmetic]

> The benefit of this approach is simplicity and predictability: when you know the rules, it's very easy to work out what Noda Time will do. The downside is that if you *don't* know the rules, it looks like it's broken. It's possible that in a future version we'll implement a "smarter" API (as a separate option, probably, rather than replacing this one) - please drop a line to the mailing list if you have requirements in this area.

[nodatime-arithmetic]: http://nodatime.org/1.1.x/userguide/arithmetic.html

## Leap seconds

Leap seconds are seconds inserted or removed at the end of certain days to synchronize UTC with the physical rotation of the Earth.  The International Earth Rotation and Reference System Services announces typically announces leap seconds six months in advance.

During leap seconds, Unix clocks become non-continuous and possibly non-monotonic.  Operating system APIs do not provide information about leap seconds and the vast majority of other APIs ignore them as well.

Google handles leap seconds internally by applying adjustment over the course of the day, resulting in a monotonic and continuous clock with 86,400 seconds per day but which has an offset of up to 500 ms from UTC.

#### References

* [United States Naval Observatory: Leap Seconds][usno-leap-seconds]
* [History of IEEE P1003.1 POSIX time][posix-time]
* [Google Official Blog: Time, technology, and leaping seconds][leap-smear]

[usno-leap-seconds]: http://tycho.usno.navy.mil/leapsec.html
[posix-time]: http://www.mail-archive.com/leapsecs@rom.usno.navy.mil/msg00109.html
[leap-smear]: http://googleblog.blogspot.com/2011/09/time-technology-and-leaping-seconds.html

## High precision timers

Each platform has its own API for high precision timers.  POSIX provides `clock_gettime()` with nanosecond resolution.  Windows provides `QueryPerformanceCounter()` and OS X provides `mach_absolute_time()`.  The timers for neither OS X nor Windows provide fixed resolution.  Instead, the resolution must be queried at runtime.

#### References

* [The Open Group Base Specifications Issue 7: Clock and timer functions][clock-gettime]
* [Apple Technical Q&A QA1398: Mach Absolute Time Units][apple-qa1398]
* [Windows Dev Center - Desktop: About Timers][msdn-timers]

[clock-gettime]: http://pubs.opengroup.org/onlinepubs/9699919799/functions/clock_gettime.html
[apple-qa1398]: https://developer.apple.com/library/mac/qa/qa1398/_index.html
[msdn-timers]: http://msdn.microsoft.com/en-us/library/windows/desktop/ms644900(v=vs.85).aspx

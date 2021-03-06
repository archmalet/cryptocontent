% Synchronization

# Basic idea
Cryptocontent is designed to work without a trusted third party by synchronizing
data over one or more encrypted files over different hosting or file sharing
providers. To allow complex objects being synchronized in this way,
Cryptocontent uses an event based log - similar to a database transaction log.

# Reasons for this approach
Our goal is to provide a mode of synchronization that is simple, fast, low on
bandwidth, as generic as possible and working with multiple devices. To achieve
these goals without using a server, we are using an event based log that is
mimicking some database functionality (i.e. create, update, delete). This
eases the solving of merge conflicts, works faster and has lower bandwidth than
using serialized objects. ***It should also allow synchronization for all
data than can be converted to json?***

# The event log
The event log consists of multiple entries.

## Entry

A single entry consists of 3 parameters and the actual data in JSON format.

### Parameters
* Timestamp
* Event type (create/update/delete)
* Object ID

## Log

Cryptocontent uses two different log files locally for performance reasons and
a single log which is shared among devices.

### Local Log
The local log consists of events on objects that have not been synchronized yet.
Their timestamp will be created once the synchronization process starts so they
will always have the latest time.

### Remote Log
The remote log holds all events that have been made to objects that have already
been synchronized with other devices (meaning objects that have been created
before our current session).

### Shared Log
The shared log is pulled from the file server or hosting provider that is used.

## Basic Synchronization
1. Clean local log to hold just the create events for the objects with all
parameters set to their latest update. Delete them from the log if they have
been deleted.
2. Merge update events in the remote log and/or remove all update events for
objects that have been deleted at a later point in the session.
3. Try to get a timed lock on the server
4. Download shared log
5. Search backwards in shared log for timestamp of last saved synchronization
event.
6. Insert remote log events line by line in the correct order, so the shared log
is correctly ordered by the timestamp. *Is this useful?*
7. Add all events from local log to the bottom of the shared log.
8. Upload shared log
9. Remove timed lock
10. Update data structure according to shared log locally.

### Example
In this example we create 2 objects locally and make updates on 3 remote objects.

#### Creation of local objects

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:object data | | |
| C 1235:object data | | |

#### Update operations on remote objects

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:object data | *TS_1* U 2236:changed params | |
| C 1235:object data | *TS_2* U 4377:changed params | |
| | *TS_3* U 1444:changed params | |
| | *TS_4* U 2236:changed params | |

#### Operations on local objects

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:object data | *TS_1* U 2236:changed params | |
| C 1235:object data | *TS_2* U 4377:changed params | |
| U 1234:changed params | *TS_3* U 1444:changed params | |
| U 1235:changed params | *TS_4* U 2236:changed params | |
| U 1234:changed params | | |
| D 1235: | | |

#### Merge Local Log
Object 1235 has been deleted at the end, so we don't need to create it. Creation
of object 1234 has been changed to reflect the later updates.

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:updated object data | *TS_1* U 2236:changed params | |
| | *TS_2* U 4377:changed params | |
| | *TS_3* U 1444:changed params | |
| | *TS_4* U 2236:changed params | |

#### Merge Remote Log

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:updated object data | *TS_2* U 4377:changed params | |
| | *TS_3* U 1444:changed params | |
| | *TS_4* U 2236:changed params | |

#### Retrieve Shared Log
*TS_S* represents the last saved timestamp of our device, so we are only
interested in lines below this line.

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:updated object data | *TS_2* U 4377:changed params | *TS_S* U 4377:changed params|
| | *TS_3* U 1444:changed params | *TS_5* U 2236:changed params |
| | *TS_4* U 2236:changed params | *TS_6* U 1515:changed params|

#### Merge Remote Log and Shared Log
Imagine the following timestamp order (smaller timestamps occurred first): TS_2
< TS_3 < TS_5 < TS_6 < TS_4

| Local Log | Remote Log | Shared Log |
|---|---|---|
| C 1234:updated object data | | *TS_S* U 4377:changed params|
| | | *TS_2* U 4377:changed params |
| | | *TS_3* U 1444:changed params |
| | | *TS_5* U 2236:changed params |
| | | *TS_6* U 1515:changed params|
| | | *TS_4* U 2236:changed params |

#### Merge Local Log and Shared Log
Timestamp order: TS_2 < TS_3 < TS_5 < TS_6 < TS_4 < TS_7

| Local Log | Remote Log | Shared Log |
|---|---|---|
| | | *TS_S* U 4377:changed params|
| | | *TS_2* U 4377:changed params |
| | | *TS_3* U 1444:changed params |
| | | *TS_5* U 2236:changed params |
| | | *TS_6* U 1515:changed params|
| | | *TS_4* U 2236:changed params |
| | | C 1234:updated object data |

## Sanitation
To keep the shared log as small as possible, the master device will regularly
sanitize the shared log. This means that a certain number of lines will be
transferred from the shared log to a backup log at certain time intervals. If
a device has not been synchronized for a long period of time it can check if it
needs to read from the backup log first by comparing its saved timestamp from
its last update with the timestamp of the backup log.

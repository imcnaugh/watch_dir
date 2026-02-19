# Elite-Dangerous-Journal-Reader

The basic high level idea is to have a watcher keep an eye on journal files that the game writes to. Each time a file is
updated, we should send that file name to the reader. The reader will keep an index of the last byte read from each file,
as well as a buffer of text, when a file is updated the reader will read to the end of the file from the last byte read,
append that to the buffer, and if a new line is found, it will removed everything before that line and send it to the
line parser.

The line will then parse the json log into a a strongly typed object and will place it on a queue for processing.

## High Level Architecture
```mermaid
---
title: High Level Architecture
---
flowchart 
    subgraph A["Elite Dangerous, the game"]
        direction LR
        ed_log_writer["Elite Dangerous Journal Writer"]
    end

    subgraph B["Journal Folder"]
        direction LR
        market_journal[Market.json]
        outfitting_journal[Outfitting.json]
        shipyard_journal[Shipyard.json]
        status_journal[Status.json]
    end

    subgraph C["Log Reader"]
        direction TB
        
        j_reader(( ))
        folder_watcher["Folder Watcher"]
        file_reader["File Reader
        
        Holds a buffer of each file, when a new line is found
        empty the buffer to that point and send that line to the line parser"]
        line_parser["Line Parser"]

        folder_watcher -->|PathBuff To Updated File| file_reader
        file_reader --> |Line, and originating file|line_parser
    end
    
    subgraph D["The Unknown"]
        direction TB
        
        your_service
    end
    
    ed_log_writer --> market_journal
    ed_log_writer --> outfitting_journal
    ed_log_writer --> shipyard_journal
    ed_log_writer --> status_journal

    folder_watcher -->|Watch files for updates| j_reader
    j_reader --> market_journal
    j_reader --> outfitting_journal
    j_reader --> shipyard_journal
    j_reader --> status_journal
    
    line_parser --> your_service
```

## TODO
Read up on Rust's mpsc
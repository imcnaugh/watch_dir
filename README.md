# Elite-Dangerous-Journal-Reader

```mermaid
---
title: High Level Architecture
---
flowchart 
    subgraph "Elite Dangerous, the game"
        direction LR
        ed_log_writer["Elite Dangerous Journal Writer"]
    end

    subgraph Journal Folder
        direction LR
        market_journal[Market.json]
        outfitting_journal[Outfitting.json]
        shipyard_journal[Shipyard.json]
        status_journal[Status.json]
    end


    ed_log_writer --> market_journal
    ed_log_writer --> outfitting_journal
    ed_log_writer --> shipyard_journal
    ed_log_writer --> status_journal
    
```
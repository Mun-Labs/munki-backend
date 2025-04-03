# Munki Backend

## Architecture Overview

### Core Domain
At the center lies our business logic and domain models - the heart of what makes our application valuable. This core remains isolated from external concerns like databases or APIs.

### Ports Layer
The middle layer defines interfaces (ports) that our core needs to interact with the outside world. These are contracts that specify what the core requires without dictating implementation details.

### Adapters Layer
The outer layer contains implementations (adapters) that fulfill the port contracts. These adapters translate between our core's needs and the specifics of external systems.

### Benefits
This architecture allows us to:
- Change external integrations without modifying core business logic
- Test business rules in isolation from external dependencies
- Develop and maintain components separately
- Easily swap implementations (e.g., switching from one crypto data provider to another)
- The arrows show how data flows through the system, with clear boundaries between components that make the system more resilient and adaptable.

### Graph

```mermaid
graph TD
    subgraph "Domain Core"
        BL[Business Logic]
        DTO[Data Transfer Objects]
        E[Entities/Models]
    end

    subgraph "Ports (Interfaces)"
        API[API Port]
        DBP[Database Port]
        CP[Crypto Data Provider Port]
        BP[Blockchain Data Port]
        MP[Market Sentiment Port]
    end

    subgraph "Adapters (Implementations)"
        RA[REST API Adapter]
        DA[Database Adapter]
        CPA[Crypto Price API Adapter]
        BPA[Blockchain Provider Adapter]
        MPA[Market Sentiment API Adapter]
    end

    subgraph "External Systems"
        Client[API Clients]
        DB[(SQL Database)]
        CryptoAPI[Crypto Price APIs]
        BlockchainAPI[Blockchain Data APIs]
        MarketAPI[Market Sentiment APIs]
    end

    %% Connections between layers
    BL --- API & DBP & CP & BP & MP
    API --- RA
    DBP --- DA
    CP --- CPA
    BP --- BPA
    MP --- MPA

    RA --- Client
    DA --- DB
    CPA --- CryptoAPI
    BPA --- BlockchainAPI
    MPA --- MarketAPI
```


## Database Design
We identified major entities and how they relate to each other and other sub-entities

![initial-design.png](docs%2Fimages%2Finitial-design.png)

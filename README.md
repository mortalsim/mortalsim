# ![MORTALSIM](img/mortalsim.png)

Welcome to MortalSim, a discreet event simulation
library for biological systems. It aims to provide
a layered, highly modular engine for realistic,
scalable simulations of living entities powered
by research-based physiological models.

## Motivation

Existing physiological simulation solutions are very
complex, computationally expensive, and/or difficult
to understand and modify. 

There is need for an adaptable software solution to
provide efficient determination of overall, high-level
physiological state which can make good use of existing,
clinically verified models.

## Goals

- Provide validated physiological events at the organism level
- Support tens to thousands of simulated organisms simultaneously in real time with minimal impact to computing resources
- Modular architecture - relatively easy integration and customization
- Support combinations of widely varying model approaches
- Make model addition / removal as easy as possible

## Potential Uses
- Epidemiological research & simulation
- Medical / Military training software
- Personalized health insights
- Gaming
- ...

## How?

### ![Layers](img/layers_shrek.jpg)

MortalSim operates under the principle that,
like ogres and onions, all living organisms can be modeled
as a set of biological layers. These layers each
provide a unique framework within which each
component can communicate and interact with other
simulation components.

MortalSim currently has the following layers:
- Core
    - `Event` subscription & dispatch.
- Circulation
    - Closed circulatory system definition.
    - Blood substance interactions with `BloodStore`s.
- Digestion
    - Ingestion, handoff, and elimination.
    - `Consumable` substance interactions.
- Nervous
    - Nervous system tree definition
    - Nerve-bound `Event` signaling

### Discrete Event Simulation

Mortalsim is centered around discrete `Event`s.
At the top level, the progression of events
provides the target simulation output, whether
that's concious level, heart rate, or any other
vital sign.

Key physiological events are provided by the core
MortalSim framework, but individual components can
define their own custom `Event`s as well.

This was selected for the following reasons:

- Simple
    - Meets the target focus of high-level overall state
    - Event driven states are easily understood and implemented
- Fast
    - Requires minimal computational resources
    - Built to be parallelizable from the ground up
- Flexible
    - Loose coupling between modules
    - Easy manipulation of system inputs and outputs

## Roadmap

### Core Functionality
#### Base
- [x] Event Trait & Framework
- [x] Substances Enums
- [x] Substance Stores
- [x] Time Manager
- [x] Sim Component Traits & Framework
- [x] Sim Component Registry
- [x] Layer Manager Traits & Framework
#### Core Layer
- [x] Core Initializer & Connector
- [x] Core Layer Logic
#### Circulation Layer
- [x] Blood Vessel Traits & Framework
- [x] Circulation Initializer & Connector
- [x] Circulation Layer Logic
#### Digestion Layer
- [x] Consumable Traits & Framework
- [x] Digestion Initializer & Connector
- [x] Digestion Layer Logic
#### Nervous Layer
- [x] Nerve Traits & Framework
- [x] Nerve Signal Traits & Framework
- [x] Nervous Initializer & Connector
- [x] Nervous Layer Logic
#### Sim
- [x] Layer Processor
- [x] Sim & Organism Traits & Framework
- [x] Sample Sim

### Basic Modules
- [x] Human Sim
- [x] Simple Blood Flow
- [ ] Cardiovascular Hemodynamics Component (IN PROGRESS)
- [ ] Respiration Component
- [ ] Brain Consciousness Component
- [ ] Kidney Component
- [ ] Liver Component
- [ ] Digestion Component(s)
- [ ] BMR Component
- [ ] Acute Injury Component(s)
- [ ] Chronic Illness Component(s)
- [ ] Musculoskeletal Components

### Module Development Tools
- [ ] SBML to MortalSim component tool (IN PROGRESS)
- [ ] Test Harness for Core Components
- [ ] Test Harness for Circulation Components
- [ ] Test Harness for Digestion Components
- [ ] Test Harness for Nervous Components

### Language Support
- [ ] Python Bindings
- [ ] JavaScript Bindings
- [ ] C++ Bindings
- [ ] Java Bindings

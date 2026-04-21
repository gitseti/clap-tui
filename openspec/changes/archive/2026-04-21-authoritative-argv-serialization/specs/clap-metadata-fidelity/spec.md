## MODIFIED Requirements

### Requirement: Value sources are surfaced clearly
The TUI SHALL distinguish between user-entered values and values sourced from defaults, environment variables, or conditional default rules. Effective values MUST be derived by parsing canonical argv with clap and inspecting clap value sources. Displaying those sources MUST NOT cause the serializer to invent argv tokens for clap-derivable values.

#### Scenario: Environment-provided value is identified as sourced
- **WHEN** an argument value comes from an environment variable
- **THEN** the form indicates that the displayed value is environment-derived
- **AND** canonical argv omits the argument unless the user explicitly emits it

#### Scenario: Conditional default is explained without pretending the user typed it
- **WHEN** clap provides a conditional default for an argument
- **THEN** the form identifies that value as default-derived rather than user-entered
- **AND** canonical argv does not materialize extra tokens solely to mirror that effective value

#### Scenario: Serialization ambiguity is distinct from validation failure
- **WHEN** invocation state cannot be serialized into unique canonical argv
- **THEN** the UI presents that as a serialization ambiguity
- **AND** it does not label the condition as a clap validation failure or derived-value source

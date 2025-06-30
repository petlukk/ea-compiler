# Eä Language - AutoGen System

This directory contains the configuration and scripts for the Eä Language Development automation system.

## Overview

The AutoGen system helps manage the development tasks and automation for the Eä Language project. It provides:

- Task management and tracking
- Automated environment setup
- Progress tracking
- Build and test automation

## Directory Structure

```
.autogen/
├── config.yaml       # Main configuration file
├── tasks/            # Individual task configurations
└── templates/        # Code generation templates
```

## Configuration

The `config.yaml` file contains the main configuration for the AutoGen system, including:

- Agent definitions and their tasks
- Project settings
- Build configuration
- Development tools
- CI/CD configuration
- Documentation settings
- Logging configuration

## Usage

### List Available Tasks

```powershell
.\autogen.ps1 list-tasks
```

### Start Working on a Task

```powershell
.\autogen.ps1 start-task LEX-001
```

### Show Project Status

```powershell
.\autogen.ps1 status
```

### Get Help

```powershell
.\autogen.ps1 help
```

## Task Lifecycle

1. **Pending**: Task is defined but not started
2. **In Progress**: Task is being worked on
3. **Review**: Task is complete and ready for review
4. **Completed**: Task has been reviewed and accepted

## Adding New Tasks

To add a new task:

1. Edit the `.autogen/config.yaml` file
2. Add a new task definition under the appropriate agent
3. Include all required fields (id, description, priority, etc.)
4. Commit the changes to version control

## Best Practices

- Keep task descriptions clear and concise
- Break large tasks into smaller, manageable subtasks
- Update task status regularly
- Document any dependencies between tasks
- Use the task ID in branch names and commit messages

## License

This project is part of the Eä Language Compiler and is licensed under the same terms.

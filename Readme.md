# Vulkan backend in rust

## Design

1. Safetly first design, removes all errors and makes sure the API cannot be used in an unsafe manner
2. Use a render graph, which is a directed acyclic graph to remove the need to do syncronization, which happens to be the most error prone part of Vulkan
3. Hide a lot of verbosity of vulkan by providing deafults and also fuctions for common use cases which the most optimal values.

## Issues

Need to include byte muck in the dependency list.

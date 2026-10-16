---
id: CORE-012
title: Resource Type Registry (Swift)
status: Done
assignee: jamiepine
parent: CORE-011
priority: High
tags: [client, swift, codegen, cache, obsolete]
depends_on: [CORE-011]
milestone: M5
last_updated: 2026-10-16
---

## Description

Create the Swift ResourceTypeRegistry that enables generic deserialization of resource events. This is the key component that makes unified events zero-friction on the client side.

## Implementation Steps

1. Define `CacheableResource` protocol
2. Create `ResourceTypeRegistry` class with decoder map
3. Implement `register<T>()` method
4. Implement `decode(resourceType:from:)` method
5. Generate registry entries from specta codegen
6. Add auto-registration on app startup
7. Integrate with event handler

## Technical Details

- Location: `packages/client-swift/Sources/WingDriveCore/Cache/ResourceTypeRegistry.swift`
- Protocol: `CacheableResource: Identifiable, Codable`
- Registry: `[String: (Data) throws -> any CacheableResource]`
- Auto-generated via specta codegen

## Example

```swift
// Protocol
protocol CacheableResource: Identifiable, Codable {
    static var resourceType: String { get }
}

// Registry
class ResourceTypeRegistry {
    private static var decoders: [String: (Data) throws -> any CacheableResource] = [:]

    static func register<T: CacheableResource>(_ type: T.Type) {
        decoders[T.resourceType] = { data in
            try JSONDecoder().decode(T.self, from: data)
        }
    }

    static func decode(resourceType: String, from data: Data) throws -> any CacheableResource {
        guard let decoder = decoders[resourceType] else {
            throw CacheError.unknownResourceType(resourceType)
        }
        return try decoder(data)
    }
}
```

## Acceptance Criteria

- [ ] ResourceTypeRegistry implemented
- [ ] All domain resources conform to CacheableResource
- [ ] Auto-registration on app init
- [ ] Error handling for unknown types
- [ ] Unit tests for registration and decoding
- [ ] Integration with EventCacheUpdater

## References

- `docs/core/events.md` lines 213-298
- `docs/core/normalized_cache.md` - Cache integration

## WingDrive triage (2026-10-16)

Obsolete, closed without implementation. The registry targets native Swift clients, and WingDrive
has no iOS or macOS Swift app: mobile is the Expo app in `apps/mobile`, which embeds the core over
FFI, and `packages/swift-client` stays a prototype. The `EventCacheUpdater` it plugs into does not
exist in this repository. Reopen as a new task if a native Swift client is ever planned.

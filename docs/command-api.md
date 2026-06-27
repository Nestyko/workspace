# AI Command API Documentation

This document defines the list of typed JSON commands supported by the AI workspace interface.

## `catalog.product.add`

**Description:** Add a product to the catalog.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "ProductAgent": {
      "properties": {
        "instructions": {
          "type": "string"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "instructions",
        "name"
      ],
      "type": "object"
    },
    "ProductKnowledgeSource": {
      "properties": {
        "label": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": [
            "string",
            "null"
          ]
        },
        "provider": {
          "type": "string"
        },
        "space": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "provider"
      ],
      "type": "object"
    },
    "ProductRoutingRule": {
      "properties": {
        "inspect_services": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "when": {
          "type": "string"
        }
      },
      "required": [
        "inspect_services",
        "when"
      ],
      "type": "object"
    },
    "ProductServices": {
      "properties": {
        "primary": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "related": {
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "primary",
        "related"
      ],
      "type": "object"
    }
  },
  "properties": {
    "agent": {
      "$ref": "#/definitions/ProductAgent"
    },
    "description": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "kind": {
      "type": "string"
    },
    "knowledge_sources": {
      "items": {
        "$ref": "#/definitions/ProductKnowledgeSource"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "routing_rules": {
      "items": {
        "$ref": "#/definitions/ProductRoutingRule"
      },
      "type": "array"
    },
    "services": {
      "$ref": "#/definitions/ProductServices"
    }
  },
  "required": [
    "agent",
    "description",
    "id",
    "kind",
    "knowledge_sources",
    "name",
    "routing_rules",
    "services"
  ],
  "title": "ProductCatalog",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "ProductAgent": {
      "properties": {
        "instructions": {
          "type": "string"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "instructions",
        "name"
      ],
      "type": "object"
    },
    "ProductKnowledgeSource": {
      "properties": {
        "label": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": [
            "string",
            "null"
          ]
        },
        "provider": {
          "type": "string"
        },
        "space": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "provider"
      ],
      "type": "object"
    },
    "ProductRoutingRule": {
      "properties": {
        "inspect_services": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "when": {
          "type": "string"
        }
      },
      "required": [
        "inspect_services",
        "when"
      ],
      "type": "object"
    },
    "ProductServices": {
      "properties": {
        "primary": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "related": {
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "primary",
        "related"
      ],
      "type": "object"
    }
  },
  "properties": {
    "agent": {
      "$ref": "#/definitions/ProductAgent"
    },
    "description": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "kind": {
      "type": "string"
    },
    "knowledge_sources": {
      "items": {
        "$ref": "#/definitions/ProductKnowledgeSource"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "routing_rules": {
      "items": {
        "$ref": "#/definitions/ProductRoutingRule"
      },
      "type": "array"
    },
    "services": {
      "$ref": "#/definitions/ProductServices"
    }
  },
  "required": [
    "agent",
    "description",
    "id",
    "kind",
    "knowledge_sources",
    "name",
    "routing_rules",
    "services"
  ],
  "title": "ProductCatalog",
  "type": "object"
}
```

---

## `catalog.product.get`

**Description:** Retrieve a product catalog by ID.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "id": {
      "type": "string"
    }
  },
  "required": [
    "id"
  ],
  "title": "CatalogGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "ProductAgent": {
      "properties": {
        "instructions": {
          "type": "string"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "instructions",
        "name"
      ],
      "type": "object"
    },
    "ProductKnowledgeSource": {
      "properties": {
        "label": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": [
            "string",
            "null"
          ]
        },
        "provider": {
          "type": "string"
        },
        "space": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "provider"
      ],
      "type": "object"
    },
    "ProductRoutingRule": {
      "properties": {
        "inspect_services": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "when": {
          "type": "string"
        }
      },
      "required": [
        "inspect_services",
        "when"
      ],
      "type": "object"
    },
    "ProductServices": {
      "properties": {
        "primary": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "related": {
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "primary",
        "related"
      ],
      "type": "object"
    }
  },
  "properties": {
    "agent": {
      "$ref": "#/definitions/ProductAgent"
    },
    "description": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "kind": {
      "type": "string"
    },
    "knowledge_sources": {
      "items": {
        "$ref": "#/definitions/ProductKnowledgeSource"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "routing_rules": {
      "items": {
        "$ref": "#/definitions/ProductRoutingRule"
      },
      "type": "array"
    },
    "services": {
      "$ref": "#/definitions/ProductServices"
    }
  },
  "required": [
    "agent",
    "description",
    "id",
    "kind",
    "knowledge_sources",
    "name",
    "routing_rules",
    "services"
  ],
  "title": "ProductCatalog",
  "type": "object"
}
```

---

## `catalog.product.list`

**Description:** List all product catalogs.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "ProductAgent": {
      "properties": {
        "instructions": {
          "type": "string"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "instructions",
        "name"
      ],
      "type": "object"
    },
    "ProductCatalog": {
      "properties": {
        "agent": {
          "$ref": "#/definitions/ProductAgent"
        },
        "description": {
          "type": "string"
        },
        "id": {
          "type": "string"
        },
        "kind": {
          "type": "string"
        },
        "knowledge_sources": {
          "items": {
            "$ref": "#/definitions/ProductKnowledgeSource"
          },
          "type": "array"
        },
        "name": {
          "type": "string"
        },
        "routing_rules": {
          "items": {
            "$ref": "#/definitions/ProductRoutingRule"
          },
          "type": "array"
        },
        "services": {
          "$ref": "#/definitions/ProductServices"
        }
      },
      "required": [
        "agent",
        "description",
        "id",
        "kind",
        "knowledge_sources",
        "name",
        "routing_rules",
        "services"
      ],
      "type": "object"
    },
    "ProductKnowledgeSource": {
      "properties": {
        "label": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": [
            "string",
            "null"
          ]
        },
        "provider": {
          "type": "string"
        },
        "space": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "provider"
      ],
      "type": "object"
    },
    "ProductRoutingRule": {
      "properties": {
        "inspect_services": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "when": {
          "type": "string"
        }
      },
      "required": [
        "inspect_services",
        "when"
      ],
      "type": "object"
    },
    "ProductServices": {
      "properties": {
        "primary": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "related": {
          "items": {
            "type": "string"
          },
          "type": "array"
        }
      },
      "required": [
        "primary",
        "related"
      ],
      "type": "object"
    }
  },
  "items": {
    "$ref": "#/definitions/ProductCatalog"
  },
  "title": "Array_of_ProductCatalog",
  "type": "array"
}
```

---

## `catalog.service.add`

**Description:** Add a service to the catalog as a separate file.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "CatalogDoc": {
      "properties": {
        "path": {
          "type": "string"
        },
        "type": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "type"
      ],
      "type": "object"
    },
    "CatalogIssueTracking": {
      "properties": {
        "component": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "project",
        "provider"
      ],
      "type": "object"
    },
    "CatalogRepo": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "name",
        "owner",
        "provider",
        "url"
      ],
      "type": "object"
    },
    "DeployConfig": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "properties": {
            "reason": {
              "type": [
                "string",
                "null"
              ]
            },
            "skip": {
              "type": "boolean"
            }
          },
          "required": [
            "skip"
          ],
          "type": "object"
        }
      ],
      "description": "Point #8 — Deploy declaration.\n\nA plain command string declares the deploy invocation. `Skip { skip: true, reason }` is the explicit opt-out for repos with no deploy target (library / CLI)."
    },
    "UnderstandAnythingConfig": {
      "description": "Point #1 — Understand-Anything artifact configuration.\n\nWhen `enabled: true`, `repo.healthcheck` expects a committed `.understand-anything/` artifact, the canonical `.gitattributes` diff-suppression lines, and the GitHub Action workflow file that refreshes the artifact on merge to the default branch.",
      "properties": {
        "enabled": {
          "type": "boolean"
        }
      },
      "required": [
        "enabled"
      ],
      "type": "object"
    }
  },
  "properties": {
    "commands": {
      "additionalProperties": {
        "type": "string"
      },
      "type": "object"
    },
    "deploy": {
      "anyOf": [
        {
          "$ref": "#/definitions/DeployConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "description": {
      "type": "string"
    },
    "docs": {
      "items": {
        "$ref": "#/definitions/CatalogDoc"
      },
      "type": "array"
    },
    "id": {
      "type": "string"
    },
    "issue_tracking": {
      "$ref": "#/definitions/CatalogIssueTracking"
    },
    "kind": {
      "type": "string"
    },
    "likely_relevant_when": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "owns": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "products": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "repo": {
      "$ref": "#/definitions/CatalogRepo"
    },
    "team": {
      "type": "string"
    },
    "understand_anything": {
      "anyOf": [
        {
          "$ref": "#/definitions/UnderstandAnythingConfig"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "required": [
    "commands",
    "description",
    "docs",
    "id",
    "issue_tracking",
    "kind",
    "likely_relevant_when",
    "name",
    "owns",
    "products",
    "repo",
    "team"
  ],
  "title": "ServiceCatalog",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "CatalogDoc": {
      "properties": {
        "path": {
          "type": "string"
        },
        "type": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "type"
      ],
      "type": "object"
    },
    "CatalogIssueTracking": {
      "properties": {
        "component": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "project",
        "provider"
      ],
      "type": "object"
    },
    "CatalogRepo": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "name",
        "owner",
        "provider",
        "url"
      ],
      "type": "object"
    },
    "DeployConfig": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "properties": {
            "reason": {
              "type": [
                "string",
                "null"
              ]
            },
            "skip": {
              "type": "boolean"
            }
          },
          "required": [
            "skip"
          ],
          "type": "object"
        }
      ],
      "description": "Point #8 — Deploy declaration.\n\nA plain command string declares the deploy invocation. `Skip { skip: true, reason }` is the explicit opt-out for repos with no deploy target (library / CLI)."
    },
    "UnderstandAnythingConfig": {
      "description": "Point #1 — Understand-Anything artifact configuration.\n\nWhen `enabled: true`, `repo.healthcheck` expects a committed `.understand-anything/` artifact, the canonical `.gitattributes` diff-suppression lines, and the GitHub Action workflow file that refreshes the artifact on merge to the default branch.",
      "properties": {
        "enabled": {
          "type": "boolean"
        }
      },
      "required": [
        "enabled"
      ],
      "type": "object"
    }
  },
  "properties": {
    "commands": {
      "additionalProperties": {
        "type": "string"
      },
      "type": "object"
    },
    "deploy": {
      "anyOf": [
        {
          "$ref": "#/definitions/DeployConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "description": {
      "type": "string"
    },
    "docs": {
      "items": {
        "$ref": "#/definitions/CatalogDoc"
      },
      "type": "array"
    },
    "id": {
      "type": "string"
    },
    "issue_tracking": {
      "$ref": "#/definitions/CatalogIssueTracking"
    },
    "kind": {
      "type": "string"
    },
    "likely_relevant_when": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "owns": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "products": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "repo": {
      "$ref": "#/definitions/CatalogRepo"
    },
    "team": {
      "type": "string"
    },
    "understand_anything": {
      "anyOf": [
        {
          "$ref": "#/definitions/UnderstandAnythingConfig"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "required": [
    "commands",
    "description",
    "docs",
    "id",
    "issue_tracking",
    "kind",
    "likely_relevant_when",
    "name",
    "owns",
    "products",
    "repo",
    "team"
  ],
  "title": "ServiceCatalog",
  "type": "object"
}
```

---

## `catalog.service.get`

**Description:** Retrieve a service catalog by ID.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "id": {
      "type": "string"
    }
  },
  "required": [
    "id"
  ],
  "title": "CatalogGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "CatalogDoc": {
      "properties": {
        "path": {
          "type": "string"
        },
        "type": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "type"
      ],
      "type": "object"
    },
    "CatalogIssueTracking": {
      "properties": {
        "component": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "project",
        "provider"
      ],
      "type": "object"
    },
    "CatalogRepo": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "name",
        "owner",
        "provider",
        "url"
      ],
      "type": "object"
    },
    "DeployConfig": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "properties": {
            "reason": {
              "type": [
                "string",
                "null"
              ]
            },
            "skip": {
              "type": "boolean"
            }
          },
          "required": [
            "skip"
          ],
          "type": "object"
        }
      ],
      "description": "Point #8 — Deploy declaration.\n\nA plain command string declares the deploy invocation. `Skip { skip: true, reason }` is the explicit opt-out for repos with no deploy target (library / CLI)."
    },
    "UnderstandAnythingConfig": {
      "description": "Point #1 — Understand-Anything artifact configuration.\n\nWhen `enabled: true`, `repo.healthcheck` expects a committed `.understand-anything/` artifact, the canonical `.gitattributes` diff-suppression lines, and the GitHub Action workflow file that refreshes the artifact on merge to the default branch.",
      "properties": {
        "enabled": {
          "type": "boolean"
        }
      },
      "required": [
        "enabled"
      ],
      "type": "object"
    }
  },
  "properties": {
    "commands": {
      "additionalProperties": {
        "type": "string"
      },
      "type": "object"
    },
    "deploy": {
      "anyOf": [
        {
          "$ref": "#/definitions/DeployConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "description": {
      "type": "string"
    },
    "docs": {
      "items": {
        "$ref": "#/definitions/CatalogDoc"
      },
      "type": "array"
    },
    "id": {
      "type": "string"
    },
    "issue_tracking": {
      "$ref": "#/definitions/CatalogIssueTracking"
    },
    "kind": {
      "type": "string"
    },
    "likely_relevant_when": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "owns": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "products": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "repo": {
      "$ref": "#/definitions/CatalogRepo"
    },
    "team": {
      "type": "string"
    },
    "understand_anything": {
      "anyOf": [
        {
          "$ref": "#/definitions/UnderstandAnythingConfig"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "required": [
    "commands",
    "description",
    "docs",
    "id",
    "issue_tracking",
    "kind",
    "likely_relevant_when",
    "name",
    "owns",
    "products",
    "repo",
    "team"
  ],
  "title": "ServiceCatalog",
  "type": "object"
}
```

---

## `catalog.service.list`

**Description:** List all service catalogs.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "CatalogDoc": {
      "properties": {
        "path": {
          "type": "string"
        },
        "type": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "type"
      ],
      "type": "object"
    },
    "CatalogIssueTracking": {
      "properties": {
        "component": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "project",
        "provider"
      ],
      "type": "object"
    },
    "CatalogRepo": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "name",
        "owner",
        "provider",
        "url"
      ],
      "type": "object"
    },
    "DeployConfig": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "properties": {
            "reason": {
              "type": [
                "string",
                "null"
              ]
            },
            "skip": {
              "type": "boolean"
            }
          },
          "required": [
            "skip"
          ],
          "type": "object"
        }
      ],
      "description": "Point #8 — Deploy declaration.\n\nA plain command string declares the deploy invocation. `Skip { skip: true, reason }` is the explicit opt-out for repos with no deploy target (library / CLI)."
    },
    "ServiceCatalog": {
      "properties": {
        "commands": {
          "additionalProperties": {
            "type": "string"
          },
          "type": "object"
        },
        "deploy": {
          "anyOf": [
            {
              "$ref": "#/definitions/DeployConfig"
            },
            {
              "type": "null"
            }
          ]
        },
        "description": {
          "type": "string"
        },
        "docs": {
          "items": {
            "$ref": "#/definitions/CatalogDoc"
          },
          "type": "array"
        },
        "id": {
          "type": "string"
        },
        "issue_tracking": {
          "$ref": "#/definitions/CatalogIssueTracking"
        },
        "kind": {
          "type": "string"
        },
        "likely_relevant_when": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "name": {
          "type": "string"
        },
        "owns": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "products": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "repo": {
          "$ref": "#/definitions/CatalogRepo"
        },
        "team": {
          "type": "string"
        },
        "understand_anything": {
          "anyOf": [
            {
              "$ref": "#/definitions/UnderstandAnythingConfig"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "commands",
        "description",
        "docs",
        "id",
        "issue_tracking",
        "kind",
        "likely_relevant_when",
        "name",
        "owns",
        "products",
        "repo",
        "team"
      ],
      "type": "object"
    },
    "UnderstandAnythingConfig": {
      "description": "Point #1 — Understand-Anything artifact configuration.\n\nWhen `enabled: true`, `repo.healthcheck` expects a committed `.understand-anything/` artifact, the canonical `.gitattributes` diff-suppression lines, and the GitHub Action workflow file that refreshes the artifact on merge to the default branch.",
      "properties": {
        "enabled": {
          "type": "boolean"
        }
      },
      "required": [
        "enabled"
      ],
      "type": "object"
    }
  },
  "items": {
    "$ref": "#/definitions/ServiceCatalog"
  },
  "title": "Array_of_ServiceCatalog",
  "type": "array"
}
```

---

## `catalog.service.update`

**Description:** Strict partial-merge patch into a service catalog entry; re-validates the catalog.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "additionalProperties": false,
  "definitions": {
    "CatalogDoc": {
      "properties": {
        "path": {
          "type": "string"
        },
        "type": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "type"
      ],
      "type": "object"
    },
    "CatalogIssueTracking": {
      "properties": {
        "component": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "project",
        "provider"
      ],
      "type": "object"
    },
    "DeployConfig": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "properties": {
            "reason": {
              "type": [
                "string",
                "null"
              ]
            },
            "skip": {
              "type": "boolean"
            }
          },
          "required": [
            "skip"
          ],
          "type": "object"
        }
      ],
      "description": "Point #8 — Deploy declaration.\n\nA plain command string declares the deploy invocation. `Skip { skip: true, reason }` is the explicit opt-out for repos with no deploy target (library / CLI)."
    },
    "UnderstandAnythingConfig": {
      "description": "Point #1 — Understand-Anything artifact configuration.\n\nWhen `enabled: true`, `repo.healthcheck` expects a committed `.understand-anything/` artifact, the canonical `.gitattributes` diff-suppression lines, and the GitHub Action workflow file that refreshes the artifact on merge to the default branch.",
      "properties": {
        "enabled": {
          "type": "boolean"
        }
      },
      "required": [
        "enabled"
      ],
      "type": "object"
    }
  },
  "description": "Strict partial-merge patch for a service catalog entry.\n\nSemantics (locked by design): - `commands` is a map → **per-key merge** (set `commands.dev` without touching `commands.test`). - Every other present field is a **top-level replace**. - `#[serde(deny_unknown_fields)]` makes unknown keys fail fast (strict mode). - After writing, `validate_catalog` is re-run; the command errors if validation fails.\n\nNote: catalog entries have no `locks.yaml` (those are per-epic workspace locks), so no lockfile is touched here.",
  "properties": {
    "commands": {
      "additionalProperties": {
        "type": "string"
      },
      "description": "Per-key merge into the existing `commands` map.",
      "type": [
        "object",
        "null"
      ]
    },
    "deploy": {
      "anyOf": [
        {
          "$ref": "#/definitions/DeployConfig"
        },
        {
          "type": "null"
        }
      ],
      "description": "Replace whole value (Point #8)."
    },
    "description": {
      "type": [
        "string",
        "null"
      ]
    },
    "docs": {
      "description": "Replace whole value.",
      "items": {
        "$ref": "#/definitions/CatalogDoc"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "id": {
      "type": "string"
    },
    "issue_tracking": {
      "anyOf": [
        {
          "$ref": "#/definitions/CatalogIssueTracking"
        },
        {
          "type": "null"
        }
      ]
    },
    "likely_relevant_when": {
      "items": {
        "type": "string"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "name": {
      "type": [
        "string",
        "null"
      ]
    },
    "owns": {
      "items": {
        "type": "string"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "products": {
      "items": {
        "type": "string"
      },
      "type": [
        "array",
        "null"
      ]
    },
    "team": {
      "type": [
        "string",
        "null"
      ]
    },
    "understand_anything": {
      "anyOf": [
        {
          "$ref": "#/definitions/UnderstandAnythingConfig"
        },
        {
          "type": "null"
        }
      ],
      "description": "Replace whole value (Point #1)."
    }
  },
  "required": [
    "id"
  ],
  "title": "CatalogServiceUpdateInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "CatalogDoc": {
      "properties": {
        "path": {
          "type": "string"
        },
        "type": {
          "type": "string"
        }
      },
      "required": [
        "path",
        "type"
      ],
      "type": "object"
    },
    "CatalogIssueTracking": {
      "properties": {
        "component": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "project",
        "provider"
      ],
      "type": "object"
    },
    "CatalogRepo": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "name",
        "owner",
        "provider",
        "url"
      ],
      "type": "object"
    },
    "DeployConfig": {
      "anyOf": [
        {
          "type": "string"
        },
        {
          "properties": {
            "reason": {
              "type": [
                "string",
                "null"
              ]
            },
            "skip": {
              "type": "boolean"
            }
          },
          "required": [
            "skip"
          ],
          "type": "object"
        }
      ],
      "description": "Point #8 — Deploy declaration.\n\nA plain command string declares the deploy invocation. `Skip { skip: true, reason }` is the explicit opt-out for repos with no deploy target (library / CLI)."
    },
    "UnderstandAnythingConfig": {
      "description": "Point #1 — Understand-Anything artifact configuration.\n\nWhen `enabled: true`, `repo.healthcheck` expects a committed `.understand-anything/` artifact, the canonical `.gitattributes` diff-suppression lines, and the GitHub Action workflow file that refreshes the artifact on merge to the default branch.",
      "properties": {
        "enabled": {
          "type": "boolean"
        }
      },
      "required": [
        "enabled"
      ],
      "type": "object"
    }
  },
  "properties": {
    "commands": {
      "additionalProperties": {
        "type": "string"
      },
      "type": "object"
    },
    "deploy": {
      "anyOf": [
        {
          "$ref": "#/definitions/DeployConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "description": {
      "type": "string"
    },
    "docs": {
      "items": {
        "$ref": "#/definitions/CatalogDoc"
      },
      "type": "array"
    },
    "id": {
      "type": "string"
    },
    "issue_tracking": {
      "$ref": "#/definitions/CatalogIssueTracking"
    },
    "kind": {
      "type": "string"
    },
    "likely_relevant_when": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    },
    "owns": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "products": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "repo": {
      "$ref": "#/definitions/CatalogRepo"
    },
    "team": {
      "type": "string"
    },
    "understand_anything": {
      "anyOf": [
        {
          "$ref": "#/definitions/UnderstandAnythingConfig"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "required": [
    "commands",
    "description",
    "docs",
    "id",
    "issue_tracking",
    "kind",
    "likely_relevant_when",
    "name",
    "owns",
    "products",
    "repo",
    "team"
  ],
  "title": "ServiceCatalog",
  "type": "object"
}
```

---

## `catalog.team.add`

**Description:** Add a team to the catalog.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "description": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "kind": {
      "type": "string"
    },
    "lead": {
      "type": [
        "string",
        "null"
      ]
    },
    "members": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    }
  },
  "required": [
    "description",
    "id",
    "kind",
    "members",
    "name"
  ],
  "title": "TeamCatalog",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "description": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "kind": {
      "type": "string"
    },
    "lead": {
      "type": [
        "string",
        "null"
      ]
    },
    "members": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    }
  },
  "required": [
    "description",
    "id",
    "kind",
    "members",
    "name"
  ],
  "title": "TeamCatalog",
  "type": "object"
}
```

---

## `catalog.team.get`

**Description:** Retrieve a team catalog by ID.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "id": {
      "type": "string"
    }
  },
  "required": [
    "id"
  ],
  "title": "CatalogGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "description": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "kind": {
      "type": "string"
    },
    "lead": {
      "type": [
        "string",
        "null"
      ]
    },
    "members": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "name": {
      "type": "string"
    }
  },
  "required": [
    "description",
    "id",
    "kind",
    "members",
    "name"
  ],
  "title": "TeamCatalog",
  "type": "object"
}
```

---

## `catalog.team.list`

**Description:** List all team catalogs.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "TeamCatalog": {
      "properties": {
        "description": {
          "type": "string"
        },
        "id": {
          "type": "string"
        },
        "kind": {
          "type": "string"
        },
        "lead": {
          "type": [
            "string",
            "null"
          ]
        },
        "members": {
          "items": {
            "type": "string"
          },
          "type": "array"
        },
        "name": {
          "type": "string"
        }
      },
      "required": [
        "description",
        "id",
        "kind",
        "members",
        "name"
      ],
      "type": "object"
    }
  },
  "items": {
    "$ref": "#/definitions/TeamCatalog"
  },
  "title": "Array_of_TeamCatalog",
  "type": "array"
}
```

---

## `catalog.validate`

**Description:** Validate all catalog files.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `context.resolve`

**Description:** Resolve products, recommended services, and knowledge sources based on a query.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "query": {
      "type": "string"
    }
  },
  "required": [
    "query"
  ],
  "title": "ContextResolveInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "ProductKnowledgeSource": {
      "properties": {
        "label": {
          "type": [
            "string",
            "null"
          ]
        },
        "project": {
          "type": [
            "string",
            "null"
          ]
        },
        "provider": {
          "type": "string"
        },
        "space": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "provider"
      ],
      "type": "object"
    },
    "RecommendedService": {
      "properties": {
        "id": {
          "type": "string"
        },
        "reason": {
          "type": "string"
        }
      },
      "required": [
        "id",
        "reason"
      ],
      "type": "object"
    }
  },
  "properties": {
    "knowledge_sources": {
      "items": {
        "$ref": "#/definitions/ProductKnowledgeSource"
      },
      "type": "array"
    },
    "products": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "query": {
      "type": "string"
    },
    "recommended_services": {
      "items": {
        "$ref": "#/definitions/RecommendedService"
      },
      "type": "array"
    }
  },
  "required": [
    "knowledge_sources",
    "products",
    "query",
    "recommended_services"
  ],
  "title": "ContextResolveOutput",
  "type": "object"
}
```

---

## `editor.open`

**Description:** Open the workspace or a specific service in an editor.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "editor": {
      "type": [
        "string",
        "null"
      ]
    },
    "epic_key": {
      "type": "string"
    },
    "service_id": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "epic_key"
  ],
  "title": "EditorOpenInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `pr.create`

**Description:** Create pull requests for changes in the workspace repositories.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "body": {
      "type": "string"
    },
    "draft": {
      "type": "boolean"
    },
    "services": {
      "items": {
        "type": "string"
      },
      "type": "array"
    },
    "title": {
      "type": "string"
    },
    "workspace_id": {
      "type": "string"
    }
  },
  "required": [
    "body",
    "draft",
    "services",
    "title",
    "workspace_id"
  ],
  "title": "PrCreateInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "prs": {
      "additionalProperties": {
        "type": "string"
      },
      "type": "object"
    },
    "workspace_id": {
      "type": "string"
    }
  },
  "required": [
    "prs",
    "workspace_id"
  ],
  "title": "PrCreateOutput",
  "type": "object"
}
```

---

## `provider.code.check_auth`

**Description:** Check authentication status with the configured code provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "authenticated": {
      "type": "boolean"
    },
    "details": {
      "type": [
        "string",
        "null"
      ]
    },
    "username": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "authenticated"
  ],
  "title": "AuthStatus",
  "type": "object"
}
```

---

## `provider.code.get_repo`

**Description:** Retrieve repository metadata details.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "name": {
      "type": "string"
    },
    "owner": {
      "type": "string"
    }
  },
  "required": [
    "name",
    "owner"
  ],
  "title": "RepoRef",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "RepoSummary": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "description": {
          "type": [
            "string",
            "null"
          ]
        },
        "full_name": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "ssh_url": {
          "type": "string"
        },
        "updated_at": {
          "type": [
            "string",
            "null"
          ]
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "full_name",
        "name",
        "owner",
        "provider",
        "ssh_url",
        "url"
      ],
      "type": "object"
    }
  },
  "properties": {
    "summary": {
      "$ref": "#/definitions/RepoSummary"
    }
  },
  "required": [
    "summary"
  ],
  "title": "RepoDetails",
  "type": "object"
}
```

---

## `provider.code.list_recent_repos`

**Description:** List recently updated repositories from the code provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "limit": {
      "format": "uint",
      "minimum": 0.0,
      "type": [
        "integer",
        "null"
      ]
    },
    "page": {
      "format": "uint",
      "minimum": 0.0,
      "type": [
        "integer",
        "null"
      ]
    }
  },
  "title": "ListRecentReposInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "RepoSummary": {
      "properties": {
        "default_branch": {
          "type": "string"
        },
        "description": {
          "type": [
            "string",
            "null"
          ]
        },
        "full_name": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        },
        "ssh_url": {
          "type": "string"
        },
        "updated_at": {
          "type": [
            "string",
            "null"
          ]
        },
        "url": {
          "type": "string"
        }
      },
      "required": [
        "default_branch",
        "full_name",
        "name",
        "owner",
        "provider",
        "ssh_url",
        "url"
      ],
      "type": "object"
    }
  },
  "items": {
    "$ref": "#/definitions/RepoSummary"
  },
  "title": "Array_of_RepoSummary",
  "type": "array"
}
```

---

## `provider.config.get_instructions`

**Description:** Retrieve custom instruction guidelines (e.g. AGENT.md equivalent) for a provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "provider_id": {
      "type": "string"
    }
  },
  "required": [
    "provider_id"
  ],
  "title": "ProviderConfigGetInstructionsInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "instructions": {
      "type": [
        "string",
        "null"
      ]
    },
    "provider_id": {
      "type": "string"
    }
  },
  "required": [
    "provider_id"
  ],
  "title": "ProviderConfigGetInstructionsOutput",
  "type": "object"
}
```

---

## `provider.config.sync_instructions`

**Description:** Regenerate the ws-managed company AGENTS.md (workflows + catalog + practices). Non-destructive to integration blocks.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "description": "Regenerate the ws-managed **company `AGENTS.md`** at the workspace root to reflect the current `workflows/*.md`, the current catalog, and company practices (incl. the repo-init healthcheck surface). Counterpart to read-only `provider.config.get_instructions`.\n\nNon-destructive to third-party integration blocks: any `<!-- BEGIN ... -->` / `<!-- END ... -->` block already present (e.g. the Beads tracker section) is preserved verbatim and re-appended.",
  "title": "ProviderConfigSyncInstructionsInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "path": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "path",
    "success"
  ],
  "title": "ProviderConfigSyncInstructionsOutput",
  "type": "object"
}
```

---

## `provider.doc.check_auth`

**Description:** Check authentication status with the configured doc provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "authenticated": {
      "type": "boolean"
    },
    "details": {
      "type": [
        "string",
        "null"
      ]
    },
    "username": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "authenticated"
  ],
  "title": "AuthStatus",
  "type": "object"
}
```

---

## `provider.doc.create_page`

**Description:** Create a new documentation page in the doc provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "body": {
      "type": "string"
    },
    "space": {
      "type": "string"
    },
    "title": {
      "type": "string"
    }
  },
  "required": [
    "body",
    "space",
    "title"
  ],
  "title": "ProviderDocCreatePageInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "page_id": {
      "type": "string"
    }
  },
  "required": [
    "page_id"
  ],
  "title": "ProviderDocCreatePageOutput",
  "type": "object"
}
```

---

## `provider.doc.get_page`

**Description:** Retrieve page content from the doc provider knowledge base.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "space": {
      "type": "string"
    },
    "title": {
      "type": "string"
    }
  },
  "required": [
    "space",
    "title"
  ],
  "title": "ProviderDocGetPageInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "content": {
      "type": "string"
    }
  },
  "required": [
    "content"
  ],
  "title": "ProviderDocGetPageOutput",
  "type": "object"
}
```

---

## `provider.doc.update_page`

**Description:** Update an existing page in the doc provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "body": {
      "type": "string"
    },
    "page_id": {
      "type": "string"
    },
    "title": {
      "type": "string"
    }
  },
  "required": [
    "body",
    "page_id",
    "title"
  ],
  "title": "ProviderDocUpdatePageInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `provider.issue.check_auth`

**Description:** Check authentication status with the configured issue provider.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "EmptyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "authenticated": {
      "type": "boolean"
    },
    "details": {
      "type": [
        "string",
        "null"
      ]
    },
    "username": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "authenticated"
  ],
  "title": "AuthStatus",
  "type": "object"
}
```

---

## `provider.issue.comment`

**Description:** Add a comment to an issue in the tracker.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "body": {
      "type": "string"
    },
    "key": {
      "type": "string"
    }
  },
  "required": [
    "body",
    "key"
  ],
  "title": "ProviderIssueCommentInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `provider.issue.create_epic`

**Description:** Create a new Epic in the issue tracker.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "description": {
      "type": [
        "string",
        "null"
      ]
    },
    "name": {
      "type": "string"
    },
    "project": {
      "type": "string"
    },
    "summary": {
      "type": "string"
    }
  },
  "required": [
    "name",
    "project",
    "summary"
  ],
  "title": "CreateEpicInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "assignee": {
      "type": [
        "string",
        "null"
      ]
    },
    "description": {
      "type": [
        "string",
        "null"
      ]
    },
    "issue_type": {
      "type": "string"
    },
    "key": {
      "type": "string"
    },
    "project_key": {
      "type": "string"
    },
    "status": {
      "type": "string"
    },
    "summary": {
      "type": "string"
    }
  },
  "required": [
    "issue_type",
    "key",
    "project_key",
    "status",
    "summary"
  ],
  "title": "Issue",
  "type": "object"
}
```

---

## `provider.issue.create_issue`

**Description:** Create a new Issue task in the issue tracker.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "description": {
      "type": [
        "string",
        "null"
      ]
    },
    "epic_key": {
      "type": [
        "string",
        "null"
      ]
    },
    "issue_type": {
      "type": "string"
    },
    "project": {
      "type": "string"
    },
    "summary": {
      "type": "string"
    }
  },
  "required": [
    "issue_type",
    "project",
    "summary"
  ],
  "title": "CreateIssueInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "assignee": {
      "type": [
        "string",
        "null"
      ]
    },
    "description": {
      "type": [
        "string",
        "null"
      ]
    },
    "issue_type": {
      "type": "string"
    },
    "key": {
      "type": "string"
    },
    "project_key": {
      "type": "string"
    },
    "status": {
      "type": "string"
    },
    "summary": {
      "type": "string"
    }
  },
  "required": [
    "issue_type",
    "key",
    "project_key",
    "status",
    "summary"
  ],
  "title": "Issue",
  "type": "object"
}
```

---

## `provider.issue.get_issue`

**Description:** Retrieve issue tracking details by key.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "key": {
      "type": "string"
    }
  },
  "required": [
    "key"
  ],
  "title": "ProviderIssueGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "assignee": {
      "type": [
        "string",
        "null"
      ]
    },
    "description": {
      "type": [
        "string",
        "null"
      ]
    },
    "issue_type": {
      "type": "string"
    },
    "key": {
      "type": "string"
    },
    "project_key": {
      "type": "string"
    },
    "status": {
      "type": "string"
    },
    "summary": {
      "type": "string"
    }
  },
  "required": [
    "issue_type",
    "key",
    "project_key",
    "status",
    "summary"
  ],
  "title": "Issue",
  "type": "object"
}
```

---

## `provider.issue.link`

**Description:** Link two issues together in the issue tracker.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "inward_key": {
      "type": "string"
    },
    "link_type": {
      "type": "string"
    },
    "outward_key": {
      "type": "string"
    }
  },
  "required": [
    "inward_key",
    "link_type",
    "outward_key"
  ],
  "title": "LinkIssuesInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `repo.fix_loop.prompt`

**Description:** Emit a harness-readable markdown spec for the 2-subagent setup fix-loop. ws does NOT run it.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "service_id": {
      "type": "string"
    }
  },
  "required": [
    "service_id"
  ],
  "title": "RepoFixLoopPromptInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "service_id": {
      "type": "string"
    },
    "spec": {
      "description": "The full markdown spec the harness follows.",
      "type": "string"
    }
  },
  "required": [
    "service_id",
    "spec"
  ],
  "title": "RepoFixLoopPromptOutput",
  "type": "object"
}
```

---

## `repo.healthcheck`

**Description:** Read-only 10-point repo-init healthcheck for a service repo (declaration + file-existence).

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "check": {
      "description": "`\"all\"` (default) or a single check id like `\"1\"`, `\"5\"`, `\"10\"`.",
      "type": [
        "string",
        "null"
      ]
    },
    "repo_path": {
      "description": "Absolute or relative path to a *working checkout* (working tree) of the repo. `ws` never clones; the harness supplies the path (e.g. a workspace worktree at `workspaces/<epic>/repos/<service_id>` or any local clone).",
      "type": "string"
    },
    "service_id": {
      "description": "Catalog service id whose repo is being healthchecked.",
      "type": "string"
    }
  },
  "required": [
    "repo_path",
    "service_id"
  ],
  "title": "RepoHealthcheckInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "CheckStatus": {
      "oneOf": [
        {
          "description": "Fully satisfied (used by file/structural + declaration gates when present).",
          "enum": [
            "present"
          ],
          "type": "string"
        },
        {
          "description": "Required gate not satisfied at all.",
          "enum": [
            "missing"
          ],
          "type": "string"
        },
        {
          "description": "Some sub-parts satisfied, others not.",
          "enum": [
            "partial"
          ],
          "type": "string"
        },
        {
          "description": "Declaration gate satisfied (field present in catalog).",
          "enum": [
            "declared"
          ],
          "type": "string"
        },
        {
          "description": "Declaration gate not satisfied (field absent from catalog).",
          "enum": [
            "not_declared"
          ],
          "type": "string"
        },
        {
          "description": "Explicitly not applicable (unused by active points today; reserved).",
          "enum": [
            "na"
          ],
          "type": "string"
        }
      ]
    },
    "HealthcheckRow": {
      "properties": {
        "blocking": {
          "description": "True when an unsatisfied status blocks repo readiness.",
          "type": "boolean"
        },
        "check_id": {
          "description": "One of \"1\"..\"8\",\"10\".",
          "type": "string"
        },
        "evidence": {
          "description": "Human-readable evidence trail (paths, present/absent keys).",
          "type": "string"
        },
        "run_hint": {
          "description": "Brief next-step hint for the harness (full templates in workflows/repo-init.md).",
          "type": "string"
        },
        "status": {
          "$ref": "#/definitions/CheckStatus"
        },
        "title": {
          "type": "string"
        }
      },
      "required": [
        "blocking",
        "check_id",
        "evidence",
        "run_hint",
        "status",
        "title"
      ],
      "type": "object"
    },
    "HealthcheckSummary": {
      "properties": {
        "blocking_failures": {
          "description": "Rows that are blocking AND unsatisfied (Present/Declared do not count).",
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        },
        "declared": {
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        },
        "missing": {
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        },
        "not_declared": {
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        },
        "partial": {
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        },
        "present": {
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        },
        "total": {
          "format": "uint",
          "minimum": 0.0,
          "type": "integer"
        }
      },
      "required": [
        "blocking_failures",
        "declared",
        "missing",
        "not_declared",
        "partial",
        "present",
        "total"
      ],
      "type": "object"
    }
  },
  "properties": {
    "repo_path": {
      "type": "string"
    },
    "rows": {
      "items": {
        "$ref": "#/definitions/HealthcheckRow"
      },
      "type": "array"
    },
    "service_id": {
      "type": "string"
    },
    "summary": {
      "$ref": "#/definitions/HealthcheckSummary"
    }
  },
  "required": [
    "repo_path",
    "rows",
    "service_id",
    "summary"
  ],
  "title": "RepoHealthcheckOutput",
  "type": "object"
}
```

---

## `repo.run`

**Description:** Deterministic single-command executor (exit or serve+poll). Never runs deploy.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "command": {
      "description": "Catalog command key to run: install | test | test_integration | agent_verify | verify_run | dev | run. `deploy` is refused.",
      "type": "string"
    },
    "repo_path": {
      "description": "Absolute or relative path to a working checkout of the repo.",
      "type": "string"
    },
    "service_id": {
      "type": "string"
    },
    "timeout": {
      "description": "Timeout in seconds (default 180).",
      "format": "uint64",
      "minimum": 0.0,
      "type": [
        "integer",
        "null"
      ]
    }
  },
  "required": [
    "command",
    "repo_path",
    "service_id"
  ],
  "title": "RepoRunInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "command": {
      "description": "Command key that was run.",
      "type": "string"
    },
    "command_value": {
      "description": "Actual shell string from the catalog.",
      "type": "string"
    },
    "duration_secs": {
      "format": "double",
      "type": "number"
    },
    "exit_code": {
      "format": "int32",
      "type": [
        "integer",
        "null"
      ]
    },
    "mode": {
      "description": "\"exit\" or \"serve\".",
      "type": "string"
    },
    "smoke_passed": {
      "description": "For exit mode: exit_code == 0. For serve mode: verify_run probe passed.",
      "type": "boolean"
    },
    "stderr_tail": {
      "type": "string"
    },
    "stdout_tail": {
      "type": "string"
    },
    "timed_out": {
      "type": "boolean"
    }
  },
  "required": [
    "command",
    "command_value",
    "duration_secs",
    "mode",
    "smoke_passed",
    "stderr_tail",
    "stdout_tail",
    "timed_out"
  ],
  "title": "RepoRunOutput",
  "type": "object"
}
```

---

## `repo.understand.verify`

**Description:** Verify Point #1 (Understand-Anything): artifact present + Action green + PR merged.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "pr_number": {
      "description": "The onboarding PR number that adds the workflow + commits the artifact.",
      "format": "uint64",
      "minimum": 0.0,
      "type": "integer"
    },
    "repo_path": {
      "description": "Absolute or relative path to a working checkout of the repo.",
      "type": "string"
    },
    "run_id": {
      "description": "Optional explicit workflow run id. If given, its conclusion is checked directly (more precise than PR-check name heuristics).",
      "format": "uint64",
      "minimum": 0.0,
      "type": [
        "integer",
        "null"
      ]
    },
    "service_id": {
      "type": "string"
    }
  },
  "required": [
    "pr_number",
    "repo_path",
    "service_id"
  ],
  "title": "RepoUnderstandVerifyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "artifact_ok": {
      "type": "boolean"
    },
    "evidence": {
      "type": "string"
    },
    "overall": {
      "type": "boolean"
    },
    "pr_merged": {
      "type": "boolean"
    },
    "service_id": {
      "type": "string"
    },
    "workflow_run_green": {
      "type": "boolean"
    }
  },
  "required": [
    "artifact_ok",
    "evidence",
    "overall",
    "pr_merged",
    "service_id",
    "workflow_run_green"
  ],
  "title": "RepoUnderstandVerifyOutput",
  "type": "object"
}
```

---

## `repo.verify`

**Description:** Deterministic run-all (post-setup); excludes deploy; stops at first failure.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "repo_path": {
      "description": "Absolute or relative path to a working checkout of the repo.",
      "type": "string"
    },
    "service_id": {
      "type": "string"
    },
    "timeout": {
      "description": "Per-command timeout in seconds (default 180).",
      "format": "uint64",
      "minimum": 0.0,
      "type": [
        "integer",
        "null"
      ]
    }
  },
  "required": [
    "repo_path",
    "service_id"
  ],
  "title": "RepoVerifyInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "VerifyStepResult": {
      "properties": {
        "command": {
          "type": "string"
        },
        "duration_secs": {
          "format": "double",
          "type": "number"
        },
        "exit_code": {
          "format": "int32",
          "type": [
            "integer",
            "null"
          ]
        },
        "passed": {
          "type": "boolean"
        },
        "stderr_tail": {
          "type": "string"
        },
        "stdout_tail": {
          "type": "string"
        },
        "timed_out": {
          "type": "boolean"
        }
      },
      "required": [
        "command",
        "duration_secs",
        "passed",
        "stderr_tail",
        "stdout_tail",
        "timed_out"
      ],
      "type": "object"
    }
  },
  "properties": {
    "all_passed": {
      "type": "boolean"
    },
    "service_id": {
      "type": "string"
    },
    "steps": {
      "items": {
        "$ref": "#/definitions/VerifyStepResult"
      },
      "type": "array"
    },
    "steps_run": {
      "format": "uint",
      "minimum": 0.0,
      "type": "integer"
    },
    "stopped_at": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "required": [
    "all_passed",
    "service_id",
    "steps",
    "steps_run"
  ],
  "title": "RepoVerifyOutput",
  "type": "object"
}
```

---

## `workspace.add_service`

**Description:** Add a service to an active implementation workspace.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "epic_key": {
      "type": "string"
    },
    "service_id": {
      "type": "string"
    }
  },
  "required": [
    "epic_key",
    "service_id"
  ],
  "title": "WorkspaceAddServiceInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `workspace.create`

**Description:** Create a per-epic multi-repo workspace.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "base_branch": {
      "type": [
        "string",
        "null"
      ]
    },
    "create_branches": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "editor": {
      "type": [
        "string",
        "null"
      ]
    },
    "epic_key": {
      "type": "string"
    },
    "services": {
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "epic_key",
    "services"
  ],
  "title": "WorkspaceCreateInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "base_branch": {
      "type": "string"
    },
    "create_branches": {
      "type": "boolean"
    },
    "editor": {
      "type": "string"
    },
    "id": {
      "type": "string"
    },
    "services": {
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "base_branch",
    "create_branches",
    "editor",
    "id",
    "services"
  ],
  "title": "Workspace",
  "type": "object"
}
```

---

## `workspace.generate_editor_files`

**Description:** Regenerate editor-specific workspace configurations.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "epic_key": {
      "type": "string"
    }
  },
  "required": [
    "epic_key"
  ],
  "title": "WorkspaceGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "message": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  },
  "required": [
    "message",
    "success"
  ],
  "title": "StatusOutput",
  "type": "object"
}
```

---

## `workspace.lock`

**Description:** Retrieve workspace lockfile details.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "epic_key": {
      "type": "string"
    }
  },
  "required": [
    "epic_key"
  ],
  "title": "WorkspaceGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "LockedRepo": {
      "properties": {
        "baseline_commit": {
          "type": "string"
        },
        "default_branch": {
          "type": "string"
        },
        "name": {
          "type": "string"
        },
        "owner": {
          "type": "string"
        },
        "provider": {
          "type": "string"
        }
      },
      "required": [
        "baseline_commit",
        "default_branch",
        "name",
        "owner",
        "provider"
      ],
      "type": "object"
    }
  },
  "properties": {
    "id": {
      "type": "string"
    },
    "repos": {
      "additionalProperties": {
        "$ref": "#/definitions/LockedRepo"
      },
      "type": "object"
    }
  },
  "required": [
    "id",
    "repos"
  ],
  "title": "WorkspaceLock",
  "type": "object"
}
```

---

## `workspace.status`

**Description:** Show status of active workspace and repositories.

### Input Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "properties": {
    "epic_key": {
      "type": "string"
    }
  },
  "required": [
    "epic_key"
  ],
  "title": "WorkspaceGetInput",
  "type": "object"
}
```

### Output Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "RepoStatus": {
      "properties": {
        "baseline_commit": {
          "type": "string"
        },
        "branch": {
          "type": "string"
        },
        "current_commit": {
          "type": "string"
        },
        "has_changes": {
          "type": "boolean"
        },
        "service_id": {
          "type": "string"
        }
      },
      "required": [
        "baseline_commit",
        "branch",
        "current_commit",
        "has_changes",
        "service_id"
      ],
      "type": "object"
    }
  },
  "properties": {
    "base_branch": {
      "type": "string"
    },
    "create_branches": {
      "type": "boolean"
    },
    "editor": {
      "type": "string"
    },
    "epic_key": {
      "type": "string"
    },
    "repo_statuses": {
      "additionalProperties": {
        "$ref": "#/definitions/RepoStatus"
      },
      "type": "object"
    },
    "services": {
      "items": {
        "type": "string"
      },
      "type": "array"
    }
  },
  "required": [
    "base_branch",
    "create_branches",
    "editor",
    "epic_key",
    "repo_statuses",
    "services"
  ],
  "title": "WorkspaceStatusOutput",
  "type": "object"
}
```

---


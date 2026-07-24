# Confluence Provider Instructions (Customer-Specific Config)

This configuration directs how the AI agent reads from and updates the Confluence wiki.

## Space Organization

### Spaces
- **ACME**: Confluence space for the Acme product.
- **PLATFORM**: Confluence space for Platform engineering.

## Document Templates

### Architecture Pages
- Save all architecture descriptions under space `ACME` or `PLATFORM` with the page title format:
  `[Architecture] <Service Name>`
- Body format must use XHTML storage format containing structural tags.

### PRDs
- Save under title: `[PRD] <Feature Name>`.

variable "owner" {
  description = "GitHub account that owns the repository"
  type        = string
  default     = "0xeae3f7c0"
}

variable "repository" {
  description = "Repository name"
  type        = string
  default     = "mono"
}

variable "required_checks" {
  description = "Check names that must pass before a pull request into main can merge"
  type        = list(string)
  default     = ["Rust checks", "Generated files"]
}

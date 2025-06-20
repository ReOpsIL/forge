use serde::{Deserialize, Serialize};

// Define profession categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProfessionCategory {
    EngineeringDevelopment,
    DesignUserExperience,
    DataAI,
    ArchitectureTechnicalLeadership,
    ProductProject,
    Security,
    InfrastructureIT,
    BusinessMarketingTech,
    OtherSpecializedRoles,
    Custom,
}

impl ProfessionCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            ProfessionCategory::EngineeringDevelopment => "Engineering & Development",
            ProfessionCategory::DesignUserExperience => "Design & User Experience",
            ProfessionCategory::DataAI => "Data & AI",
            ProfessionCategory::ArchitectureTechnicalLeadership => "Architecture & Technical Leadership",
            ProfessionCategory::ProductProject => "Product & Project",
            ProfessionCategory::Security => "Security",
            ProfessionCategory::InfrastructureIT => "Infrastructure & IT",
            ProfessionCategory::BusinessMarketingTech => "Business & Marketing Tech",
            ProfessionCategory::OtherSpecializedRoles => "Other Specialized Roles",
            ProfessionCategory::Custom => "Custom Roles",
        }
    }

    pub fn all_categories() -> Vec<ProfessionCategory> {
        vec![
            ProfessionCategory::EngineeringDevelopment,
            ProfessionCategory::DesignUserExperience,
            ProfessionCategory::DataAI,
            ProfessionCategory::ArchitectureTechnicalLeadership,
            ProfessionCategory::ProductProject,
            ProfessionCategory::Security,
            ProfessionCategory::InfrastructureIT,
            ProfessionCategory::BusinessMarketingTech,
            ProfessionCategory::OtherSpecializedRoles,
            ProfessionCategory::Custom,
        ]
    }
}

// Define profession struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profession {
    pub id: String,
    pub name: String,
    pub category: ProfessionCategory,
    pub prompts: ProfessionPrompts,
}

// Define prompts for each profession
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfessionPrompts {
    pub auto_complete_system_prompt: String,
    pub auto_complete_user_prompt: String,
    pub enhance_description_system_prompt: String,
    pub enhance_description_user_prompt: String,
    pub generate_tasks_system_prompt: String,
    pub generate_tasks_user_prompt: String,
    pub process_markdown_spec_system_prompt: String,
    pub process_markdown_spec_user_prompt: String,
}

// Function to get all professions with their default prompts
pub fn get_all_professions() -> Vec<Profession> {
    vec![
        // Engineering & Development
        Profession {
            id: "frontend_developer".to_string(),
            name: "Frontend Developer".to_string(),
            category: ProfessionCategory::EngineeringDevelopment,
            prompts: create_frontend_developer_prompts(),
        },
        Profession {
            id: "backend_developer".to_string(),
            name: "Backend Developer".to_string(),
            category: ProfessionCategory::EngineeringDevelopment,
            prompts: create_backend_developer_prompts(),
        },
        Profession {
            id: "fullstack_developer".to_string(),
            name: "Full-Stack Developer".to_string(),
            category: ProfessionCategory::EngineeringDevelopment,
            prompts: create_fullstack_developer_prompts(),
        },
        Profession {
            id: "mobile_developer".to_string(),
            name: "Mobile Developer".to_string(),
            category: ProfessionCategory::EngineeringDevelopment,
            prompts: create_mobile_developer_prompts(),
        },
        Profession {
            id: "devops_engineer".to_string(),
            name: "DevOps Engineer".to_string(),
            category: ProfessionCategory::EngineeringDevelopment,
            prompts: create_devops_engineer_prompts(),
        },

        // Design & User Experience
        Profession {
            id: "ui_designer".to_string(),
            name: "UI Designer".to_string(),
            category: ProfessionCategory::DesignUserExperience,
            prompts: create_ui_designer_prompts(),
        },
        Profession {
            id: "ux_designer".to_string(),
            name: "UX Designer".to_string(),
            category: ProfessionCategory::DesignUserExperience,
            prompts: create_ux_designer_prompts(),
        },

        // Data & AI
        Profession {
            id: "data_scientist".to_string(),
            name: "Data Scientist".to_string(),
            category: ProfessionCategory::DataAI,
            prompts: create_data_scientist_prompts(),
        },
        Profession {
            id: "ml_engineer".to_string(),
            name: "Machine Learning Engineer".to_string(),
            category: ProfessionCategory::DataAI,
            prompts: create_ml_engineer_prompts(),
        },

        // Architecture & Technical Leadership
        Profession {
            id: "software_architect".to_string(),
            name: "Software Architect".to_string(),
            category: ProfessionCategory::ArchitectureTechnicalLeadership,
            prompts: create_software_architect_prompts(),
        },
        Profession {
            id: "tech_lead".to_string(),
            name: "Technical Lead".to_string(),
            category: ProfessionCategory::ArchitectureTechnicalLeadership,
            prompts: create_tech_lead_prompts(),
        },

        // Product & Project
        Profession {
            id: "product_manager".to_string(),
            name: "Product Manager".to_string(),
            category: ProfessionCategory::ProductProject,
            prompts: create_product_manager_prompts(),
        },
        Profession {
            id: "project_manager".to_string(),
            name: "Project Manager".to_string(),
            category: ProfessionCategory::ProductProject,
            prompts: create_project_manager_prompts(),
        },

        // Security
        Profession {
            id: "security_engineer".to_string(),
            name: "Security Engineer".to_string(),
            category: ProfessionCategory::Security,
            prompts: create_security_engineer_prompts(),
        },

        // Infrastructure & IT
        Profession {
            id: "cloud_engineer".to_string(),
            name: "Cloud Engineer".to_string(),
            category: ProfessionCategory::InfrastructureIT,
            prompts: create_cloud_engineer_prompts(),
        },

        // Business & Marketing Tech
        Profession {
            id: "growth_engineer".to_string(),
            name: "Growth Engineer".to_string(),
            category: ProfessionCategory::BusinessMarketingTech,
            prompts: create_growth_engineer_prompts(),
        },

        // Other Specialized Roles
        Profession {
            id: "technical_writer".to_string(),
            name: "Technical Writer".to_string(),
            category: ProfessionCategory::OtherSpecializedRoles,
            prompts: create_technical_writer_prompts(),
        },

        // Custom Roles
        Profession {
            id: "custom_role_1".to_string(),
            name: "Custom Role 1".to_string(),
            category: ProfessionCategory::Custom,
            prompts: create_custom_prompts(),
        },

        Profession {
            id: "custom_role_2".to_string(),
            name: "Custom Role 2".to_string(),
            category: ProfessionCategory::Custom,
            prompts: create_custom_prompts(),
        },

        Profession {
            id: "custom_role_3".to_string(),
            name: "Custom Role 3".to_string(),
            category: ProfessionCategory::Custom,
            prompts: create_custom_prompts(),
        },
        Profession {
            id: "custom_role_4".to_string(),
            name: "Custom Role 4".to_string(),
            category: ProfessionCategory::Custom,
            prompts: create_custom_prompts(),
        },
        Profession {
            id: "custom_role_5".to_string(),
            name: "Custom Role 5".to_string(),
            category: ProfessionCategory::Custom,
            prompts: create_custom_prompts(),
        },
    ]
}

// Function to get a profession by ID
pub fn get_profession_by_id(id: &str) -> Option<Profession> {
    get_all_professions().into_iter().find(|p| p.id == id)
}

// Default prompts for each profession
fn create_frontend_developer_prompts() -> ProfessionPrompts {
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior frontend developer specializing in UI/UX implementation, JavaScript frameworks, and responsive design. Your role is to complete partial component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial frontend component description by adding 2-3 sentences that:
1. Clarify the UI/UX implementation approach
2. Specify key frameworks, libraries or techniques involved
3. Highlight any important responsive design or accessibility considerations

Maintain the original intent and technical level. Be specific about technologies like React, Vue, Angular, CSS frameworks, or state management when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in frontend development documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that frontend developers can directly use for coding.".to_string(),
        enhance_description_user_prompt: "Transform the following frontend component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and UI/UX requirements
- Technical implementation approach using modern frontend frameworks
- Component structure, props, and state management
- Styling approach and responsive design considerations
- Accessibility requirements
- Performance considerations
- Testing approach

**Guidelines:**
- Use precise technical language for frontend development
- Include specific technologies/frameworks (React, Vue, Angular, etc.)
- Ensure the description is actionable for frontend developers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior frontend developer and project manager expert at breaking down UI components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by frontend developers.".to_string(),
        generate_tasks_user_prompt: "
# JSON Task Generation Prompt
**IMPORTANT: You must respond with valid JSON only. No additional text, explanations, or markdown formatting.**
Based on the frontend component description below, generate a prioritized list of concrete implementation tasks

**JSON Schema:**
```json
{
  \"component_name\": \"string\",
  \"total_tasks\": number,
  \"tasks\": [
    {
      \"task_id\": string,
      \"task_name\": \"string\",
      \"description\": \"string\", 
      \"acceptance_criteria\": [
        \"string\"
      ],
      \"dependencies\": [
        \"string or task_id\"
      ],
      \"estimated_effort\": \"S|M|L\",
      \"files_affected\": [
        \"string\"
      ],
      \"function_signatures\": [
        \"string\"
      ],
      \"testing_requirements\": [
        \"string\"
      ],
      \"log\": \"\",
      \"commit_id\": \"\",
      \"status\": \"[TODO]\", 
    }
  ]
}
```

**Task Requirements:**
- Frontend-specific and actionable (avoid vague terms)
- Include component structure, styling, and interactivity tasks
- Estimable in scope (typically 1-8 hours of work)
- Include relevant file names, component names, or code locations
- Specify testing requirements including unit and UI tests
- Indicate dependencies between tasks using task IDs or descriptive names
- Use effort indicators: S (Simple, 1-3 hours), M (Medium, 3-6 hours), L (Large, 6-8 hours)
- Task ID: task_id should be a random alpha numeric string of 6 characters.

**Component Description:**
{}

**Output Requirements:**
- Return ONLY valid JSON
- No explanatory text before or after the JSON
- Ensure all JSON syntax is correct
- Include 5-15 prioritized tasks
- Tasks should be ordered by implementation priority".to_string(),
        process_markdown_spec_system_prompt: "You are a frontend architecture analyst expert at parsing technical specifications and extracting structured UI component implementation details. Your output must be valid JSON that can be directly consumed by frontend development tools.".to_string(),
        process_markdown_spec_user_prompt: "Analyze the following frontend technical specification markdown and extract structured implementation blocks for UI components. 

**Output Requirements:**
- Valid JSON array format
- Each block must have clear, implementable descriptions for UI components
- Inputs/outputs should specify props, events, and state
- Include styling, accessibility, and responsive design requirements
- Ensure naming follows frontend component conventions
- Block ID: block_id should be a random alpha numeric string of 6 characters.

**JSON Schema:**
```json
{
  \"name\": \"CamelCaseComponentName\",
  \"block_id\": \"sg3gf6\",
  \"description\": \"Detailed implementation description with technical specifics for this UI component\",
  \"inputs\": [
    {\"name\": \"propName\", \"ctype\": \"dataType\", \"description\": \"purpose and format of this prop\"}
  ],
  \"outputs\": [
    {\"name\": \"eventName\", \"ctype\": \"eventType\", \"description\": \"expected event behavior\"}
  ],
  \"dependencies\": [\"RequiredComponent1\", \"RequiredLibrary2\"]
}
```

**Analysis Guidelines:**
- Extract only implementable UI components (ignore documentation sections)
- Infer missing technical details from context
- Group related UI elements into logical components
- Ensure each component is self-contained where possible

Specification document:
{}
".to_string(),
    }
}

fn create_backend_developer_prompts() -> ProfessionPrompts {
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior backend developer specializing in API design, database architecture, and server-side performance. Your role is to complete partial backend component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial backend component description by adding 2-3 sentences that:
1. Clarify the technical implementation approach for server-side functionality
2. Specify key APIs, database structures, or algorithms involved
3. Highlight any important performance, security, or scalability considerations

Maintain the original intent and technical level. Be specific about technologies, frameworks, or database systems when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in backend development documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that backend developers can directly use for coding.".to_string(),
        enhance_description_user_prompt: "Transform the following backend component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and scope of the backend service
- Technical implementation approach using appropriate server-side technologies
- API endpoints, request/response formats
- Database schema and data models
- Authentication and authorization requirements
- Error handling and logging strategy
- Performance and scalability considerations

**Guidelines:**
- Use precise technical language for backend development
- Include specific technologies/frameworks (Node.js, Django, Spring, etc.)
- Ensure the description is actionable for backend developers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior backend developer and project manager expert at breaking down server-side components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by backend developers.".to_string(),
        generate_tasks_user_prompt: "
# JSON Task Generation Prompt
**IMPORTANT: You must respond with valid JSON only. No additional text, explanations, or markdown formatting.**
Based on the backend component description below, generate a prioritized list of concrete implementation tasks

**JSON Schema:**
```json
{
  \"component_name\": \"string\",
  \"total_tasks\": number,
  \"tasks\": [
    {
      \"task_id\": string,
      \"task_name\": \"string\",
      \"description\": \"string\", 
      \"acceptance_criteria\": [
        \"string\"
      ],
      \"dependencies\": [
        \"string or task_id\"
      ],
      \"estimated_effort\": \"S|M|L\",
      \"files_affected\": [
        \"string\"
      ],
      \"function_signatures\": [
        \"string\"
      ],
      \"testing_requirements\": [
        \"string\"
      ],
      \"log\": \"\",
      \"commit_id\": \"\",
      \"status\": \"[TODO]\", 
    }
  ]
}
```

**Task Requirements:**
- Backend-specific and actionable (avoid vague terms)
- Include API endpoints, database operations, and business logic tasks
- Estimable in scope (typically 1-8 hours of work)
- Include relevant file names, function names, or code locations
- Specify testing requirements including unit and integration tests
- Indicate dependencies between tasks using task IDs or descriptive names
- Use effort indicators: S (Simple, 1-3 hours), M (Medium, 3-6 hours), L (Large, 6-8 hours)
- Task ID: task_id should be a random alpha numeric string of 6 characters.

**Component Description:**
{}

**Output Requirements:**
- Return ONLY valid JSON
- No explanatory text before or after the JSON
- Ensure all JSON syntax is correct
- Include 5-15 prioritized tasks
- Tasks should be ordered by implementation priority".to_string(),
        process_markdown_spec_system_prompt: "You are a backend architecture analyst expert at parsing technical specifications and extracting structured server-side implementation details. Your output must be valid JSON that can be directly consumed by backend development tools.".to_string(),
        process_markdown_spec_user_prompt: "Analyze the following backend technical specification markdown and extract structured implementation blocks for server-side components. 

**Output Requirements:**
- Valid JSON array format
- Each block must have clear, implementable descriptions for backend services
- Inputs/outputs should specify API endpoints, request/response formats
- Include database operations, authentication, and error handling requirements
- Ensure naming follows backend service conventions
- Block ID: block_id should be a random alpha numeric string of 6 characters.

**JSON Schema:**
```json
{
  \"name\": \"CamelCaseServiceName\",
  \"block_id\": \"sg3gf6\",
  \"description\": \"Detailed implementation description with technical specifics for this backend service\",
  \"inputs\": [
    {\"name\": \"endpointPath\", \"ctype\": \"requestType\", \"description\": \"purpose and format of this API endpoint\"}
  ],
  \"outputs\": [
    {\"name\": \"responseName\", \"ctype\": \"responseType\", \"description\": \"expected response format\"}
  ],
  \"dependencies\": [\"RequiredService1\", \"RequiredDatabase2\"]
}
```

**Analysis Guidelines:**
- Extract only implementable backend services (ignore documentation sections)
- Infer missing technical details from context
- Group related API endpoints into logical services
- Ensure each service is self-contained where possible

Specification document:
{}
".to_string(),
    }
}

// Create similar functions for other professions
fn create_fullstack_developer_prompts() -> ProfessionPrompts {
    // Default prompts with fullstack focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior fullstack developer with expertise in both frontend and backend technologies. Your role is to complete partial software component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial fullstack component description by adding 2-3 sentences that:
1. Clarify the technical implementation approach for both frontend and backend aspects
2. Specify key interfaces, APIs, or data structures involved
3. Highlight any important considerations for integration between frontend and backend

Maintain the original intent and technical level. Be specific about technologies, frameworks, or architectural patterns when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in fullstack development documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that cover both frontend and backend aspects.".to_string(),
        enhance_description_user_prompt: "Transform the following fullstack component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and scope
- Frontend implementation approach (UI/UX, frameworks, state management)
- Backend implementation approach (APIs, services, database)
- Data flow between frontend and backend
- Key interfaces and data structures
- Authentication and security considerations
- Performance and scalability aspects

**Guidelines:**
- Use precise technical language
- Include specific technologies/frameworks for both frontend and backend
- Ensure the description is actionable for developers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior fullstack developer and project manager expert at breaking down software components into granular, executable development tasks covering both frontend and backend aspects.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a fullstack architecture analyst expert at parsing technical specifications and extracting structured implementation components for both frontend and backend systems.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_mobile_developer_prompts() -> ProfessionPrompts {
    // Default prompts with mobile development focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior mobile developer specializing in native and cross-platform app development. Your role is to complete partial mobile component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial mobile component description by adding 2-3 sentences that:
1. Clarify the technical implementation approach for mobile platforms
2. Specify key UI components, APIs, or data structures involved
3. Highlight any important platform-specific or performance considerations

Maintain the original intent and technical level. Be specific about technologies like iOS/Swift, Android/Kotlin, React Native, or Flutter when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in mobile app development documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that mobile developers can directly use for coding.".to_string(),
        enhance_description_user_prompt: "Transform the following mobile component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and scope
- Technical implementation approach for target mobile platforms
- UI/UX implementation details
- Data management and state handling
- Platform-specific considerations
- Performance and battery optimization
- Testing approach for mobile

**Guidelines:**
- Use precise technical language for mobile development
- Include specific technologies/frameworks (iOS/Swift, Android/Kotlin, React Native, Flutter)
- Ensure the description is actionable for mobile developers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior mobile developer and project manager expert at breaking down mobile app components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by mobile developers.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a mobile architecture analyst expert at parsing technical specifications and extracting structured implementation components for mobile applications. Your output must be valid JSON that can be directly consumed by mobile development tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_devops_engineer_prompts() -> ProfessionPrompts {
    // Default prompts with DevOps focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior DevOps engineer specializing in CI/CD pipelines, infrastructure as code, and cloud services. Your role is to complete partial DevOps component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial DevOps component description by adding 2-3 sentences that:
1. Clarify the technical implementation approach for automation or infrastructure
2. Specify key tools, services, or configurations involved
3. Highlight any important security, scalability, or reliability considerations

Maintain the original intent and technical level. Be specific about technologies like Docker, Kubernetes, AWS/Azure/GCP, or CI/CD tools when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in DevOps and infrastructure documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that DevOps engineers can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following DevOps component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and scope
- Technical implementation approach using appropriate DevOps tools
- Infrastructure as code specifications
- CI/CD pipeline details
- Monitoring and alerting strategy
- Security considerations
- Disaster recovery and high availability plans

**Guidelines:**
- Use precise technical language for DevOps and infrastructure
- Include specific technologies/tools (Docker, Kubernetes, Terraform, etc.)
- Ensure the description is actionable for DevOps engineers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior DevOps engineer and project manager expert at breaking down infrastructure and automation components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by DevOps engineers.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a DevOps architecture analyst expert at parsing technical specifications and extracting structured implementation components for infrastructure and automation. Your output must be valid JSON that can be directly consumed by DevOps tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

// Create similar functions for other professions
fn create_ui_designer_prompts() -> ProfessionPrompts {
    // Default prompts with UI design focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior UI designer specializing in visual design systems, interface components, and design-to-code implementation. Your role is to complete partial UI design descriptions with precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial UI design description by adding 2-3 sentences that:
1. Clarify the visual design approach and aesthetic direction
2. Specify key UI components, patterns, or design system elements involved
3. Highlight any important considerations for consistency, accessibility, or responsive design

Maintain the original intent and design language. Be specific about design tools, frameworks, or methodologies when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in UI design documentation. Transform brief design descriptions into comprehensive, implementation-ready specifications that designers and developers can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following UI design description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and scope of the UI component or screen
- Visual design approach including color, typography, and spacing
- Component states and variations
- Responsive behavior across device sizes
- Accessibility requirements
- Animation and interaction details
- Design-to-code implementation notes

**Guidelines:**
- Use precise design language
- Include specific design system references or component libraries
- Ensure the description is actionable for both designers and developers
- Maintain focus on visual and interaction details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior UI designer and project manager expert at breaking down design components into granular, executable design and implementation tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by designers and frontend developers.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a UI design analyst expert at parsing design specifications and extracting structured implementation components for interface elements. Your output must be valid JSON that can be directly consumed by design and development tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_ux_designer_prompts() -> ProfessionPrompts {
    // Default prompts with UX design focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior UX designer specializing in user research, information architecture, and interaction design. Your role is to complete partial UX design descriptions with precision while maintaining clarity and user-centricity.".to_string(),
        auto_complete_user_prompt: "Complete the following partial UX design description by adding 2-3 sentences that:
1. Clarify the user-centered approach and interaction model
2. Specify key user flows, information architecture, or interaction patterns involved
3. Highlight any important considerations for usability, accessibility, or user testing

Maintain the original intent and design thinking. Be specific about UX methodologies, research techniques, or design principles when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in UX design documentation. Transform brief experience design descriptions into comprehensive, implementation-ready specifications that UX designers and product teams can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following UX design description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and user needs being addressed
- User research insights and personas
- User journey and flow diagrams
- Information architecture considerations
- Interaction patterns and behaviors
- Usability and accessibility requirements
- Success metrics and testing approach

**Guidelines:**
- Use precise UX terminology
- Include specific methodologies and research techniques
- Ensure the description is actionable for UX designers and product teams
- Maintain focus on user-centered design principles

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior UX designer and project manager expert at breaking down experience design into granular, executable research and design tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by UX professionals.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a UX design analyst expert at parsing experience specifications and extracting structured implementation components for user flows and interactions. Your output must be valid JSON that can be directly consumed by design and product teams.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

// Create similar functions for other professions
fn create_data_scientist_prompts() -> ProfessionPrompts {
    // Default prompts with data science focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior data scientist specializing in statistical analysis, machine learning, and data visualization. Your role is to complete partial data science component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial data science component description by adding 2-3 sentences that:
1. Clarify the analytical approach or machine learning methodology
2. Specify key algorithms, data structures, or visualization techniques involved
3. Highlight any important considerations for data quality, model performance, or interpretability

Maintain the original intent and technical level. Be specific about statistical methods, ML frameworks, or data processing techniques when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in data science documentation. Transform brief analytical component descriptions into comprehensive, implementation-ready specifications that data scientists can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following data science component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and analytical objectives
- Data requirements and preprocessing steps
- Methodology and algorithm selection
- Feature engineering approach
- Model training and evaluation strategy
- Performance metrics and validation techniques
- Deployment and monitoring considerations

**Guidelines:**
- Use precise statistical and machine learning terminology
- Include specific libraries, frameworks, and tools (Python, R, TensorFlow, etc.)
- Ensure the description is actionable for data scientists
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior data scientist and project manager expert at breaking down analytical components into granular, executable data science tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by data scientists.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a data science architecture analyst expert at parsing analytical specifications and extracting structured implementation components for data processing and modeling. Your output must be valid JSON that can be directly consumed by data science tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_ml_engineer_prompts() -> ProfessionPrompts {
    // Default prompts with ML engineering focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior machine learning engineer specializing in model development, training pipelines, and ML systems. Your role is to complete partial ML component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial machine learning component description by adding 2-3 sentences that:
1. Clarify the ML implementation approach or model architecture
2. Specify key frameworks, training procedures, or deployment strategies involved
3. Highlight any important considerations for model performance, scalability, or monitoring

Maintain the original intent and technical level. Be specific about ML frameworks, model architectures, or MLOps practices when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in machine learning engineering documentation. Transform brief ML component descriptions into comprehensive, implementation-ready specifications that ML engineers can directly use for coding.".to_string(),
        enhance_description_user_prompt: "Transform the following machine learning component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and ML objectives
- Model architecture and design decisions
- Data pipeline and preprocessing requirements
- Training procedure and hyperparameter tuning
- Evaluation metrics and validation strategy
- Deployment architecture and serving infrastructure
- Monitoring and maintenance plan

**Guidelines:**
- Use precise machine learning terminology
- Include specific frameworks and tools (TensorFlow, PyTorch, MLflow, etc.)
- Ensure the description is actionable for ML engineers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior machine learning engineer and project manager expert at breaking down ML components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by ML engineers.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a machine learning architecture analyst expert at parsing ML specifications and extracting structured implementation components for model development and deployment. Your output must be valid JSON that can be directly consumed by ML engineering tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_software_architect_prompts() -> ProfessionPrompts {
    // Default prompts with software architecture focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior software architect specializing in system design, architectural patterns, and technical strategy. Your role is to complete partial architecture descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial software architecture description by adding 2-3 sentences that:
1. Clarify the architectural approach or design pattern
2. Specify key components, interfaces, or integration points involved
3. Highlight any important considerations for scalability, maintainability, or performance

Maintain the original intent and technical level. Be specific about architectural patterns, system boundaries, or technical constraints when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in software architecture documentation. Transform brief architectural descriptions into comprehensive, implementation-ready specifications that development teams can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following software architecture description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and system scope
- Architectural style and patterns
- Component decomposition and responsibilities
- Interface definitions and contracts
- Data flow and state management
- Non-functional requirements (performance, security, scalability)
- Technology stack and implementation constraints

**Guidelines:**
- Use precise architectural terminology
- Include specific design patterns and principles
- Ensure the description is actionable for development teams
- Maintain focus on system-level design decisions

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior software architect and project manager expert at breaking down architectural components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by development teams.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a software architecture analyst expert at parsing technical specifications and extracting structured implementation components for system design. Your output must be valid JSON that can be directly consumed by architectural design tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_tech_lead_prompts() -> ProfessionPrompts {
    // Default prompts with technical leadership focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior technical lead specializing in team leadership, technical decision-making, and software delivery. Your role is to complete partial component descriptions with technical precision while considering implementation feasibility and team capabilities.".to_string(),
        auto_complete_user_prompt: "Complete the following partial component description by adding 2-3 sentences that:
1. Clarify the technical approach and implementation strategy
2. Specify key technical decisions, trade-offs, or team coordination points
3. Highlight any important considerations for delivery timeline, quality, or team skills

Maintain the original intent and technical level. Be specific about technologies, team structure, or delivery constraints when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in technical leadership documentation. Transform brief component descriptions into comprehensive, implementation-ready specifications that development teams can directly use while considering team capabilities and project constraints.".to_string(),
        enhance_description_user_prompt: "Transform the following component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and business value
- Technical implementation approach with team considerations
- Component breakdown with ownership assignments
- Technical dependencies and integration points
- Quality requirements and testing strategy
- Timeline considerations and delivery milestones
- Risk assessment and mitigation strategies

**Guidelines:**
- Use precise technical language balanced with team-oriented guidance
- Include specific technologies and implementation approaches
- Ensure the description is actionable for the development team
- Maintain focus on both technical details and delivery aspects

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior technical lead and project manager expert at breaking down software components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly assigned to team members with appropriate skills.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a technical leadership analyst expert at parsing specifications and extracting structured implementation components while considering team capabilities and project constraints. Your output must be valid JSON that can be directly consumed by project management tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_product_manager_prompts() -> ProfessionPrompts {
    // Default prompts with product management focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior product manager specializing in product strategy, feature definition, and user-centered design. Your role is to complete partial product feature descriptions with precision while maintaining clarity and business value.".to_string(),
        auto_complete_user_prompt: "Complete the following partial product feature description by adding 2-3 sentences that:
1. Clarify the user value proposition and business objectives
2. Specify key user interactions, workflows, or experience elements
3. Highlight any important considerations for market fit, user adoption, or success metrics

Maintain the original intent and product vision. Be specific about user needs, market positioning, or strategic goals when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in product management documentation. Transform brief feature descriptions into comprehensive, implementation-ready specifications that product and development teams can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following product feature description into a detailed, professional specification that includes:

**Required elements:**
- Clear user problem statement and value proposition
- User stories and acceptance criteria
- Feature scope and boundaries
- User experience flow and interactions
- Technical considerations and constraints
- Success metrics and KPIs
- Rollout strategy and phasing

**Guidelines:**
- Use precise product management terminology
- Include specific user scenarios and personas
- Ensure the description is actionable for both product and development teams
- Maintain focus on user value and business outcomes

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior product manager expert at breaking down product features into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by development teams while maintaining alignment with product goals.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a product management analyst expert at parsing product specifications and extracting structured implementation components that balance user needs with technical feasibility. Your output must be valid JSON that can be directly consumed by product and development tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_project_manager_prompts() -> ProfessionPrompts {
    // Default prompts with project management focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior project manager specializing in software delivery, team coordination, and project planning. Your role is to complete partial project component descriptions with precision while maintaining clarity and deliverability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial project component description by adding 2-3 sentences that:
1. Clarify the delivery approach and project management methodology
2. Specify key milestones, dependencies, or resource requirements
3. Highlight any important considerations for timeline, budget, or risk management

Maintain the original intent and project scope. Be specific about delivery methodologies, team structure, or project constraints when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in project management documentation. Transform brief project component descriptions into comprehensive, implementation-ready specifications that project teams can directly use for planning and execution.".to_string(),
        enhance_description_user_prompt: "Transform the following project component description into a detailed, professional specification that includes:

**Required elements:**
- Clear project objectives and success criteria
- Scope definition and boundaries
- Delivery approach and methodology
- Timeline and key milestones
- Resource requirements and team structure
- Risk assessment and mitigation strategies
- Dependencies and critical path considerations

**Guidelines:**
- Use precise project management terminology
- Include specific methodologies and frameworks (Agile, Scrum, Kanban, etc.)
- Ensure the description is actionable for project teams
- Maintain focus on delivery planning and execution

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior project manager expert at breaking down project components into granular, executable tasks with clear ownership and timelines. Focus on creating tasks that are specific, measurable, and can be directly assigned to team members.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a project management analyst expert at parsing project specifications and extracting structured implementation components with clear timelines and dependencies. Your output must be valid JSON that can be directly consumed by project management tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_security_engineer_prompts() -> ProfessionPrompts {
    // Default prompts with security engineering focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior security engineer specializing in application security, threat modeling, and secure coding practices. Your role is to complete partial security component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial security component description by adding 2-3 sentences that:
1. Clarify the security approach or protection mechanism
2. Specify key security controls, protocols, or frameworks involved
3. Highlight any important considerations for threat mitigation, compliance, or security testing

Maintain the original intent and technical level. Be specific about security standards, attack vectors, or defensive techniques when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in security engineering documentation. Transform brief security component descriptions into comprehensive, implementation-ready specifications that security engineers can directly use for implementation.".to_string(),
        enhance_description_user_prompt: "Transform the following security component description into a detailed, professional specification that includes:

**Required elements:**
- Clear security objectives and threat model
- Technical implementation approach for security controls
- Authentication and authorization mechanisms
- Data protection and encryption requirements
- Security testing and validation approach
- Compliance considerations and standards
- Incident response and monitoring strategy

**Guidelines:**
- Use precise security terminology
- Include specific security frameworks and standards (OWASP, NIST, etc.)
- Ensure the description is actionable for security engineers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior security engineer and project manager expert at breaking down security components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by security and development teams.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a security architecture analyst expert at parsing technical specifications and extracting structured security implementation components. Your output must be valid JSON that can be directly consumed by security engineering tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_cloud_engineer_prompts() -> ProfessionPrompts {
    // Default prompts with cloud engineering focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior cloud engineer specializing in cloud architecture, infrastructure as code, and managed services. Your role is to complete partial cloud component descriptions with technical precision while maintaining clarity and implementability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial cloud component description by adding 2-3 sentences that:
1. Clarify the cloud implementation approach or service architecture
2. Specify key cloud services, infrastructure patterns, or deployment methods involved
3. Highlight any important considerations for scalability, cost optimization, or cloud security

Maintain the original intent and technical level. Be specific about cloud providers (AWS, Azure, GCP), IaC tools, or service configurations when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in cloud engineering documentation. Transform brief cloud component descriptions into comprehensive, implementation-ready specifications that cloud engineers can directly use for deployment.".to_string(),
        enhance_description_user_prompt: "Transform the following cloud component description into a detailed, professional specification that includes:

**Required elements:**
- Clear purpose and cloud architecture overview
- Technical implementation using specific cloud services
- Infrastructure as code approach
- Networking and security configuration
- Scalability and high availability design
- Cost optimization strategy
- Monitoring and operational considerations

**Guidelines:**
- Use precise cloud terminology
- Include specific cloud providers and services (AWS, Azure, GCP)
- Ensure the description is actionable for cloud engineers
- Maintain focus on implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior cloud engineer and project manager expert at breaking down cloud infrastructure components into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by cloud engineers.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a cloud architecture analyst expert at parsing technical specifications and extracting structured implementation components for cloud infrastructure. Your output must be valid JSON that can be directly consumed by cloud engineering tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_growth_engineer_prompts() -> ProfessionPrompts {
    // Default prompts with growth engineering focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior growth engineer specializing in user acquisition, engagement metrics, and growth experimentation. Your role is to complete partial growth component descriptions with technical precision while maintaining clarity and business impact.".to_string(),
        auto_complete_user_prompt: "Complete the following partial growth engineering component description by adding 2-3 sentences that:
1. Clarify the growth strategy or experimentation approach
2. Specify key metrics, tracking methods, or technical implementations involved
3. Highlight any important considerations for user behavior, conversion optimization, or A/B testing

Maintain the original intent and technical level. Be specific about analytics tools, growth frameworks, or experimentation methodologies when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a technical writing expert specializing in growth engineering documentation. Transform brief growth component descriptions into comprehensive, implementation-ready specifications that growth engineers can directly use for implementation.".to_string(),
        enhance_description_user_prompt: "Transform the following growth engineering component description into a detailed, professional specification that includes:

**Required elements:**
- Clear growth objectives and target metrics
- Technical implementation approach for tracking and experimentation
- User funnel and conversion points
- A/B testing methodology and statistical approach
- Analytics integration and data collection
- Success criteria and evaluation framework
- Rollout and iteration strategy

**Guidelines:**
- Use precise growth and analytics terminology
- Include specific tools and platforms (Google Analytics, Optimizely, etc.)
- Ensure the description is actionable for growth engineers
- Maintain focus on measurable outcomes and implementation details

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior growth engineer and project manager expert at breaking down growth initiatives into granular, executable development tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by growth and development teams.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a growth engineering analyst expert at parsing specifications and extracting structured implementation components for user acquisition and engagement. Your output must be valid JSON that can be directly consumed by growth engineering tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}

fn create_technical_writer_prompts() -> ProfessionPrompts {
    // Default prompts with technical writing focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior technical writer specializing in software documentation, API references, and user guides. Your role is to complete partial documentation descriptions with precision while maintaining clarity and usability.".to_string(),
        auto_complete_user_prompt: "Complete the following partial documentation description by adding 2-3 sentences that:
1. Clarify the documentation approach or content structure
2. Specify key documentation elements, formats, or audience considerations
3. Highlight any important considerations for clarity, completeness, or technical accuracy

Maintain the original intent and technical level. Be specific about documentation types, tools, or standards when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a documentation expert specializing in technical writing for software products. Transform brief documentation descriptions into comprehensive, implementation-ready specifications that technical writers can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following documentation description into a detailed, professional specification that includes:

**Required elements:**
- Clear documentation purpose and target audience
- Content structure and organization
- Documentation types and formats
- Technical accuracy requirements
- Style guide and terminology standards
- Visual elements and code examples
- Publication and maintenance strategy

**Guidelines:**
- Use precise technical writing terminology
- Include specific documentation tools and platforms
- Ensure the description is actionable for technical writers
- Maintain focus on user comprehension and information architecture

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior technical writer and project manager expert at breaking down documentation projects into granular, executable writing tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by documentation teams.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a documentation analyst expert at parsing product specifications and extracting structured documentation requirements. Your output must be valid JSON that can be directly consumed by documentation management tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}
fn create_custom_prompts() -> ProfessionPrompts {
    // Default prompts with technical writing focus
    ProfessionPrompts {
        auto_complete_system_prompt: "You are a senior _______ specializing in ______, ______, and ______. Your role is to ______.".to_string(),
        auto_complete_user_prompt: "Complete the following partial ______  description by adding 2-3 sentences that:
1. Clarify the ______ approach or ______
2. Specify key ______ elements, ______, or ______
3. Highlight any important considerations for clarity, completeness, or ______ accuracy

Maintain the original intent and ______ ______. Be specific about ______ types, tools, or ______ when relevant.

Partial description:
{}

".to_string(),
        enhance_description_system_prompt: "You are a ______ expert specializing in ______ ______ for ______ ______. Transform brief ______ descriptions into comprehensive, ______ ready ______ that ______ can directly use.".to_string(),
        enhance_description_user_prompt: "Transform the following ______ description into a detailed, professional ______ that includes:

**Required elements:**
- Clear documentation purpose and target audience
- Content structure and organization
- Documentation types and formats
- Technical accuracy requirements
- Style guide and terminology standards
- Visual elements and code examples
- Publication and maintenance strategy

**Guidelines:**
- Use precise technical writing terminology
- Include specific documentation tools and platforms
- Ensure the description is actionable for technical writers
- Maintain focus on user comprehension and information architecture

Original description:
{}

".to_string(),
        generate_tasks_system_prompt: "You are a senior ______ ______ and ______ expert at breaking down ______ projects into granular, e______ tasks. Focus on creating tasks that are specific, measurable, and can be directly implemented by ______ teams.".to_string(),
        generate_tasks_user_prompt: crate::project_config::DEFAULT_GENERATE_TASKS_USER_PROMPT.to_string(),
        process_markdown_spec_system_prompt: "You are a ______ ______ expert at ______ ______ ______ and extracting structured ______ requirements. Your output must be valid JSON that can be directly consumed by ______ ______ tools.".to_string(),
        process_markdown_spec_user_prompt: crate::project_config::DEFAULT_PROCESS_MARKDOWN_SPEC_USER_PROMPT.to_string(),
    }
}



// Get default prompts based on profession ID
pub fn get_default_prompts(profession_id: Option<&str>) -> ProfessionPrompts {
    match profession_id {
        Some(id) => {
            if let Some(profession) = get_profession_by_id(id) {
                profession.prompts
            } else {
                // Default to software architect if profession not found
                create_software_architect_prompts()
            }
        },
        None => create_software_architect_prompts(), // Default to software architect
    }
}

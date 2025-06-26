### **Enhanced Technical Specification: "WordQuest" - An English Learning Game for Hebrew-Speaking 5th Graders**

#### **1. Purpose and Scope**

**1.1. Purpose**
This document specifies the technical requirements for "WordQuest," a browser-based educational game designed to teach foundational English to 5th-grade Hebrew-speaking students. The core pedagogical goal is to foster both **receptive language skills** (recognizing words and their meanings) and introductory **productive skills** (pronunciation and simple sentence formation) through a gamified, engaging experience.

The application is designed for the **student persona**: a 10-11 year old child, comfortable with digital interfaces but requiring clear, localized (Hebrew) instructions and immediate, positive feedback to maintain engagement.

**1.2. Scope**

**1.2.1. In-Scope Features (v1.0):**
*   **Game Engine:** A client-side engine that loads and sequences learning modules based on a predefined JSON configuration.
*   **Learning Modules:**
    *   **Interactive Flashcards:** A module for introducing new vocabulary. Users can click to flip the card (showing the Hebrew translation) and click an icon to hear the English pronunciation.
    *   **Multiple-Choice Quiz:** A module that tests vocabulary by asking the user to match a Hebrew word/image to one of four English options.
    *   **Drag-and-Drop Matching Game:** A module where users drag English word labels to their corresponding images.
*   **Content:**
    *   A curated vocabulary set of at least 100 words covering core topics (animals, food, numbers, colors, household items).
    *   Simple sentence structures (e.g., "I see a...", "This is a...").
*   **User Interface:**
    *   A fully responsive UI, optimized for tablet (primary) and desktop use.
    *   All instructions, menus, and feedback messages will be in Hebrew.
*   **Technology Stack Summary:**
    *   **Frontend:** React 18+ (with Hooks)
    *   **Build Tool:** Vite
    *   **UI Library:** Material-UI (MUI) v5
    *   **Internationalization:** i18next
    *   **Audio:** Web Speech API

**1.2.2. Out-of-Scope Features (for v1.0):**
*   User authentication, accounts, or profiles.
*   Persistent progress tracking across sessions.
*   A backend server or database.
*   Multiplayer or competitive features.
*   Advanced grammar, free-form text input, or speech recognition.
*   Support for mobile phone screen sizes (the focus is on larger tablet/desktop screens).

---

#### **2. Technical Implementation Approach**

**2.1. Frontend Architecture**
*   **Framework:** **React 18+** using functional components and hooks exclusively. A clear separation will be maintained between **container components** (handling logic, state, and data fetching) and **presentational components** (receiving props and rendering UI).
*   **Build Tool:** **Vite** will be used for its fast development server with HMR and optimized production builds.
*   **Styling:** **Material-UI (MUI) v5** will be the core UI library.
    *   A custom theme will be defined in `src/theme.js` to standardize colors, typography (using a Hebrew-friendly font like 'Heebo'), and component spacing, ensuring a consistent look and feel.
    *   Styling will be implemented using MUI's `sx` prop for instance-specific styles and the `styled()` utility for creating reusable, styled components.
*   **Project Structure:** The codebase will be organized as follows to ensure maintainability:
    ```
    /src
    ├── /assets
    │   ├── /images
    │   └── /sounds
    ├── /components
    │   ├── /common         # Reusable components (e.g., Button, Modal)
    │   ├── /game           # Game-specific components (e.g., QuizView, Flashcard)
    │   └── /layout         # Main layout components (e.g., Header, Footer)
    ├── /contexts           # React Context for state management (e.g., GameContext.js)
    ├── /data               # JSON data files for content (vocabulary.json)
    ├── /hooks              # Custom hooks (e.g., useSpeechSynthesizer.js)
    ├── /locales
    │   ├── /en/translation.json
    │   └── /he/translation.json
    ├── /services           # Data loading logic
    ├── App.jsx
    ├── main.jsx
    └── theme.js
    ```

**2.2. State Management**
*   Global game state (e.g., `currentScore`, `level`, `activeModule`, `userAnswers`) will be managed by a combination of `useReducer` and `useContext`.
*   A `GameContext` will be created to provide the state and dispatch function to all components within the game. This avoids prop-drilling and centralizes game logic.
*   **Example State Object:**
    ```javascript
    {
      gameState: 'playing', // 'menu', 'playing', 'level_complete'
      currentLevel: 1,
      score: 0,
      lives: 3,
      currentQuestionIndex: 0,
      questions: [], // Loaded from JSON for the current level
    }
    ```

**2.3. Internationalization (i18n)**
*   **`i18next`** and **`react-i18next`** will be configured to load translation files from the `/public/locales/{lang}/translation.json` directory.
*   The application will initialize with Hebrew (`he`) as the default language. All UI text will be rendered using the `t()` translation function.

**2.4. Audio**
*   A custom hook, `useSpeechSynthesizer`, will be created to encapsulate the **Web Speech API** logic.
*   **Functionality:** The hook will expose a `speak(text)` function.
*   **Error Handling:** It will gracefully handle cases where the API is not supported by the browser. It will also attempt to find a suitable English voice and log a warning if one is not available. UI elements that trigger speech will be disabled if the API is unavailable.

---

#### **3. Key Interfaces, APIs, and Data Structures**

**3.1. Data Structures (in `src/data/`)**
*   **`vocabulary.json`**: The master list of all learnable words.
    ```json
    {
      "word001": {
        "english": "Cat",
        "hebrew": "חתול",
        "category": "animals",
        "image": "/assets/images/cat.webp",
        "sentence": "This is a cat."
      },
      ...
    }
    ```
*   **`levels.json`**: Defines the structure and content of each level.
    ```json
    [
      {
        "level": 1,
        "title": "Animals",
        "modules": [
          { "type": "flashcards", "wordIds": ["word001", "word002", ...] },
          { "type": "quiz", "questionIds": ["q001", "q002", ...] }
        ]
      },
      ...
    ]
    ```
*   **`quizzes.json`**: A repository of all quiz questions.
    ```json
    {
      "q001": {
        "type": "multiple-choice-text",
        "prompt": "What is 'חתול' in English?",
        "wordId": "word001",
        "options": ["Dog", "Cat", "Bird", "Fish"], // Options are strings
        "correctAnswer": "Cat"
      },
      "q002": {
        "type": "multiple-choice-image",
        "prompt": "Which one is the 'Dog'?",
        "correctWordId": "word003",
        "optionWordIds": ["word001", "word003", "word005", "word008"] // Options are word IDs to fetch images
      }
    }
    ```

**3.2. Component Interfaces (React Props)**
*   **`<GameEngine levelConfig={...} onGameComplete={...} />`**:
    *   `levelConfig` (object, required): A level object from `levels.json`.
    *   `onGameComplete` (function, required): Callback triggered when the level is finished, passing the final score.
*   **`<QuizView question={...} onAnswerSubmit={...} />`**:
    *   `question` (object, required): A question object from `quizzes.json`.
    *   `onAnswerSubmit` (function, required): Callback that passes the user's selected answer.
*   **`<FeedbackIndicator isCorrect={...} />`**:
    *   `isCorrect` (boolean | null): `true` displays positive feedback, `false` displays negative, `null` hides it. Animates in and out.

**3.3. API Interfaces (Custom Hooks)**
*   **`useSpeechSynthesizer()`**:
    *   **Returns:** `{ speak: (text: string) => void, isSupported: boolean }`

---

#### **4. Input/Output Specifications**

**4.1. Inputs**
*   **User Events:**
    *   `onClick` on buttons, answer choices, flashcards, and audio icons.
    *   `onDragStart`, `onDragOver`, `onDrop` for the matching game.
*   **Data Loading:**
    *   Asynchronous `fetch` requests to load `/data/*.json` files when the application starts or a new level begins.

**4.2. Outputs**
*   **UI State Changes:**
    *   Rendering of the current game module (flashcard, quiz, etc.) based on the `GameContext` state.
    *   Dynamic updates to the score and lives display.
    *   CSS-based animations and transitions for feedback (e.g., a selected answer choice flashes green for correct, red for incorrect).
    *   A summary modal is displayed upon level completion, showing the score and a "Next Level" or "Play Again" button.
*   **Audio Cues:**
    *   English word pronunciation via the Web Speech API on user request.
    *   A positive sound effect (`/assets/sounds/correct.mp3`) on correct answers.
    *   A negative sound effect (`/assets/sounds/incorrect.mp3`) on incorrect answers.

---

#### **5. Constraints, Dependencies, and Assumptions**

**5.1. Constraints**
*   **Performance:** The initial application load time must be under 3 seconds on a standard broadband connection. Image assets must be optimized (e.g., `.webp` format, max width of 1024px).
*   **Accessibility:** The application must adhere to WCAG 2.1 Level AA guidelines, including sufficient color contrast ratios (4.5:1 for normal text) and keyboard navigability for all interactive elements.
*   **Browser Support:** Must be fully functional on the latest two stable versions of Google Chrome, Mozilla Firefox, and Apple Safari on desktop and tablet devices.

**5.2. Dependencies**
*   **NPM Packages:**
    *   `react`, `react-dom`: Core React library.
    *   `vite`: Build and development tooling.
    *   `@mui/material`, `@mui/icons-material`, `@emotion/react`, `@emotion/styled`: For UI components and styling.
    *   `i18next`, `react-i18next`, `i18next-http-backend`: For internationalization.
    *   `prop-types`: For runtime prop validation during development.

**5.3. Assumptions**
*   All required vocabulary, sentences, and quiz content will be provided and validated by an educational expert before development begins.
*   All image and sound assets will be provided in web-optimized formats and will be royalty-free or properly licensed for this use case.
*   The target users have a stable internet connection required for the initial asset loading.

---

#### **6. Success Criteria & Acceptance Conditions**

*   **AC1: Application Loading**
    *   **GIVEN** a user navigates to the application's root URL
    *   **THEN** the main menu screen is rendered within 3 seconds
    *   **AND** all UI text (e.g., "Start Game") is displayed in Hebrew.

*   **AC2: Flashcard Module**
    *   **GIVEN** a user starts a level with a flashcard module
    *   **WHEN** the user clicks on a flashcard
    *   **THEN** the card flips to reveal the Hebrew translation.
    *   **AND WHEN** the user clicks the audio icon
    *   **THEN** the English word is pronounced clearly.

*   **AC3: Quiz Module - Correct Answer**
    *   **GIVEN** a user is presented with a multiple-choice question
    *   **WHEN** they select the correct answer
    *   **THEN** the selected option is highlighted with a green border and a "Correct!" message is shown
    *   **AND** a positive sound effect is played
    *   **AND** their score is incremented.

*   **AC4: Quiz Module - Incorrect Answer**
    *   **GIVEN** a user is presented with a multiple-choice question
    *   **WHEN** they select an incorrect answer
    *   **THEN** the selected option is highlighted with a red border and an "Incorrect" message is shown
    *   **AND** a negative sound effect is played
    *   **AND** the number of lives is decremented.

*   **AC5: Responsiveness**
    *   **GIVEN** the application is viewed on a device
    *   **WHEN** the screen width is between 768px and 1920px
    *   **THEN** all UI elements are visible, legible, and usable without horizontal scrolling.

*   **AC6: Accessibility**
    *   **GIVEN** a user is interacting with the application
    *   **WHEN** they use the `Tab` key
    *   **THEN** they can navigate through all interactive elements (buttons, links, answers) in a logical order.

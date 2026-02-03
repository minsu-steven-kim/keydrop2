// Login form detection utilities

export interface LoginForm {
  form: HTMLFormElement | null;
  usernameField: HTMLInputElement | null;
  passwordField: HTMLInputElement | null;
}

// Common username field selectors
const USERNAME_SELECTORS = [
  'input[type="email"]',
  'input[type="text"][name*="user"]',
  'input[type="text"][name*="email"]',
  'input[type="text"][name*="login"]',
  'input[type="text"][id*="user"]',
  'input[type="text"][id*="email"]',
  'input[type="text"][id*="login"]',
  'input[autocomplete="username"]',
  'input[autocomplete="email"]',
];

// Common password field selectors
const PASSWORD_SELECTORS = [
  'input[type="password"]',
  'input[autocomplete="current-password"]',
  'input[autocomplete="new-password"]',
];

// Detect login forms on the page
export function detectLoginForms(): LoginForm[] {
  const forms: LoginForm[] = [];

  // First, try to find password fields
  const passwordFields = document.querySelectorAll<HTMLInputElement>(
    PASSWORD_SELECTORS.join(', ')
  );

  for (const passwordField of passwordFields) {
    // Skip hidden fields
    if (!isVisible(passwordField)) continue;

    // Find associated form
    const form = passwordField.closest('form');

    // Find username field
    let usernameField: HTMLInputElement | null = null;

    if (form) {
      // Look within the form first
      usernameField = findUsernameField(form);
    } else {
      // Look for nearby username field
      usernameField = findNearbyUsernameField(passwordField);
    }

    forms.push({
      form,
      usernameField,
      passwordField,
    });
  }

  return forms;
}

// Find username field within a container
function findUsernameField(
  container: HTMLElement
): HTMLInputElement | null {
  for (const selector of USERNAME_SELECTORS) {
    const field = container.querySelector<HTMLInputElement>(selector);
    if (field && isVisible(field)) {
      return field;
    }
  }

  // Fallback: find any visible text input before password
  const textInputs = container.querySelectorAll<HTMLInputElement>(
    'input[type="text"], input:not([type])'
  );
  for (const input of textInputs) {
    if (isVisible(input) && !input.name?.toLowerCase().includes('search')) {
      return input;
    }
  }

  return null;
}

// Find username field near password field (for forms without <form> tags)
function findNearbyUsernameField(
  passwordField: HTMLInputElement
): HTMLInputElement | null {
  // Try common parent containers
  let parent = passwordField.parentElement;
  let depth = 0;
  const maxDepth = 5;

  while (parent && depth < maxDepth) {
    const usernameField = findUsernameField(parent);
    if (usernameField) {
      return usernameField;
    }
    parent = parent.parentElement;
    depth++;
  }

  return null;
}

// Check if element is visible
function isVisible(element: HTMLElement): boolean {
  if (!element) return false;

  const style = window.getComputedStyle(element);
  if (style.display === 'none' || style.visibility === 'hidden') {
    return false;
  }

  const rect = element.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

// Check if current page is a login page
export function isLoginPage(): boolean {
  const forms = detectLoginForms();
  return forms.length > 0;
}

// Get the primary login form (first visible one)
export function getPrimaryLoginForm(): LoginForm | null {
  const forms = detectLoginForms();
  return forms.length > 0 ? forms[0] : null;
}

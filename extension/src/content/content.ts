// Content script - runs on every page

import { detectLoginForms, isLoginPage } from './detector';
import { fillLoginForm, showAutofillIcon, showCredentialDropdown } from './autofill';

// State
let pageCredentials: Array<{
  id: string;
  name: string;
  username: string;
  password: string;
}> = [];

// Initialize content script
function init() {
  // Check if this is a login page
  if (!isLoginPage()) return;

  // Request credentials for this URL
  chrome.runtime.sendMessage(
    { type: 'GET_ITEMS_FOR_URL', url: window.location.href },
    (response) => {
      if (response?.success && response.data?.length > 0) {
        pageCredentials = response.data;
        setupAutofillIcons();
      }
    }
  );

  // Watch for dynamically added login forms
  const observer = new MutationObserver(() => {
    if (isLoginPage() && pageCredentials.length > 0) {
      setupAutofillIcons();
    }
  });

  observer.observe(document.body, {
    childList: true,
    subtree: true,
  });
}

// Setup autofill icons on detected login forms
function setupAutofillIcons() {
  const forms = detectLoginForms();

  forms.forEach((form) => {
    // Add icon to username field
    if (form.usernameField) {
      showAutofillIcon(form.usernameField, () => {
        showDropdown(form.usernameField!);
      });
    }

    // Add icon to password field
    if (form.passwordField) {
      showAutofillIcon(form.passwordField, () => {
        showDropdown(form.passwordField!);
      });
    }
  });
}

// Show credential dropdown
function showDropdown(field: HTMLInputElement) {
  const credentialOptions = pageCredentials.map((c) => ({
    id: c.id,
    name: c.name,
    username: c.username,
  }));

  showCredentialDropdown(field, credentialOptions, (id) => {
    const credential = pageCredentials.find((c) => c.id === id);
    if (credential) {
      fillLoginForm(credential.username, credential.password);
    }
  });
}

// Listen for messages from background script
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === 'AUTOFILL') {
    const success = fillLoginForm(message.username, message.password);
    sendResponse({ success });
  }
  return true;
});

// Run when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}

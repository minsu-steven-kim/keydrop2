// Autofill functionality

import { detectLoginForms, type LoginForm } from './detector';

// Fill a login form with credentials
export function fillLoginForm(
  username: string,
  password: string,
  form?: LoginForm
): boolean {
  const targetForm = form || detectLoginForms()[0];

  if (!targetForm) {
    console.log('Keydrop: No login form found');
    return false;
  }

  let filled = false;

  // Fill username
  if (targetForm.usernameField) {
    fillField(targetForm.usernameField, username);
    filled = true;
  }

  // Fill password
  if (targetForm.passwordField) {
    fillField(targetForm.passwordField, password);
    filled = true;
  }

  return filled;
}

// Fill a single field with proper event triggering
function fillField(field: HTMLInputElement, value: string): void {
  // Focus the field
  field.focus();

  // Clear existing value
  field.value = '';

  // Set the new value
  field.value = value;

  // Trigger input events (important for React, Angular, Vue etc.)
  const inputEvent = new Event('input', { bubbles: true, cancelable: true });
  field.dispatchEvent(inputEvent);

  const changeEvent = new Event('change', { bubbles: true, cancelable: true });
  field.dispatchEvent(changeEvent);

  // For React specifically
  const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    'value'
  )?.set;
  if (nativeInputValueSetter) {
    nativeInputValueSetter.call(field, value);
    field.dispatchEvent(new Event('input', { bubbles: true }));
  }

  // Blur the field
  field.blur();
}

// Create and show the autofill icon on a field
export function showAutofillIcon(
  field: HTMLInputElement,
  onClick: () => void
): HTMLElement {
  // Remove existing icon if present
  const existingIcon = field.parentElement?.querySelector('.keydrop-autofill-icon');
  if (existingIcon) {
    existingIcon.remove();
  }

  // Create icon element
  const icon = document.createElement('div');
  icon.className = 'keydrop-autofill-icon';
  icon.innerHTML = '🔐';
  icon.title = 'Keydrop: Click to autofill';

  // Position the icon
  icon.style.cssText = `
    position: absolute;
    right: 8px;
    top: 50%;
    transform: translateY(-50%);
    cursor: pointer;
    font-size: 16px;
    z-index: 10000;
    opacity: 0.7;
    transition: opacity 0.2s;
  `;

  // Make parent relative if not already
  const parent = field.parentElement;
  if (parent) {
    const parentStyle = window.getComputedStyle(parent);
    if (parentStyle.position === 'static') {
      parent.style.position = 'relative';
    }
    parent.appendChild(icon);
  }

  // Add hover effect
  icon.addEventListener('mouseenter', () => {
    icon.style.opacity = '1';
  });
  icon.addEventListener('mouseleave', () => {
    icon.style.opacity = '0.7';
  });

  // Add click handler
  icon.addEventListener('click', (e) => {
    e.preventDefault();
    e.stopPropagation();
    onClick();
  });

  return icon;
}

// Show dropdown with credential options
export function showCredentialDropdown(
  field: HTMLInputElement,
  credentials: Array<{ id: string; name: string; username: string }>,
  onSelect: (id: string) => void
): HTMLElement {
  // Remove existing dropdown
  const existingDropdown = document.querySelector('.keydrop-dropdown');
  if (existingDropdown) {
    existingDropdown.remove();
  }

  // Create dropdown
  const dropdown = document.createElement('div');
  dropdown.className = 'keydrop-dropdown';

  const rect = field.getBoundingClientRect();
  dropdown.style.cssText = `
    position: fixed;
    left: ${rect.left}px;
    top: ${rect.bottom + 4}px;
    width: ${Math.max(rect.width, 250)}px;
    max-height: 200px;
    overflow-y: auto;
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 10001;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  `;

  // Add header
  const header = document.createElement('div');
  header.style.cssText = `
    padding: 8px 12px;
    border-bottom: 1px solid #eee;
    font-size: 12px;
    color: #666;
    display: flex;
    align-items: center;
    gap: 6px;
  `;
  header.innerHTML = '🔐 <span>Keydrop</span>';
  dropdown.appendChild(header);

  // Add credential items
  credentials.forEach((cred) => {
    const item = document.createElement('div');
    item.className = 'keydrop-dropdown-item';
    item.style.cssText = `
      padding: 10px 12px;
      cursor: pointer;
      border-bottom: 1px solid #f0f0f0;
      transition: background 0.2s;
    `;
    item.innerHTML = `
      <div style="font-weight: 500; color: #333; margin-bottom: 2px;">${escapeHtml(cred.name)}</div>
      <div style="font-size: 12px; color: #666;">${escapeHtml(cred.username)}</div>
    `;

    item.addEventListener('mouseenter', () => {
      item.style.background = '#f5f5f5';
    });
    item.addEventListener('mouseleave', () => {
      item.style.background = 'transparent';
    });
    item.addEventListener('click', () => {
      onSelect(cred.id);
      dropdown.remove();
    });

    dropdown.appendChild(item);
  });

  // Add to document
  document.body.appendChild(dropdown);

  // Close on click outside
  const closeHandler = (e: MouseEvent) => {
    if (!dropdown.contains(e.target as Node)) {
      dropdown.remove();
      document.removeEventListener('click', closeHandler);
    }
  };
  setTimeout(() => {
    document.addEventListener('click', closeHandler);
  }, 100);

  return dropdown;
}

// Escape HTML to prevent XSS
function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

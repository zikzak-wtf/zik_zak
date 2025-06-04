#!/usr/bin/env python3
"""
Quick health check to verify realistic product recipes are loaded.
"""

import requests
import json

SERVER_URL = "http://localhost:3003"

def check_recipes():
    """Check if our realistic product recipes are loaded."""
    try:
        response = requests.get(f"{SERVER_URL}/recipes")
        if response.status_code == 200:
            recipes = response.json()

            print("🦖 Recipe Status Check")
            print("=" * 30)

            realistic_recipes = [name for name in recipes.keys() if "realistic" in name]

            if realistic_recipes:
                print("✅ Realistic product recipes found:")
                for name in realistic_recipes:
                    recipe = recipes[name]
                    print(f"   📝 {name}")
                    print(f"      Description: {recipe.get('description', 'No description')}")
                    print(f"      Inputs: {len(recipe.get('inputs', []))} parameters")
                    print(f"      Operations: {recipe.get('operations_count', 0)} steps")
                    print()

                print("🚀 Ready to test! Run: python3 test_realistic_product.py")
                return True
            else:
                print("❌ Realistic product recipes not found!")
                print("   Available recipes:", list(recipes.keys()))
                print("   📝 Please restart the server to load new recipes:")
                print("   🔄 Stop the server (Ctrl+C) and run: cargo run")
                return False
        else:
            print(f"❌ Failed to connect: {response.status_code}")
            return False

    except requests.exceptions.ConnectionError:
        print("❌ Server not running on port 3003. Start with: cargo run")
        return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    check_recipes()

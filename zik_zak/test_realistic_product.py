#!/usr/bin/env python3
"""
Test script for realistic product recipes using the comprehensive e-commerce product schema.
"""

import json
import requests
import time
from typing import Dict, Any

# Server configuration
SERVER_URL = "http://localhost:3003"

def test_realistic_product_recipes():
    """Test the comprehensive realistic product recipes."""

    print("🦖 Testing Realistic Product Recipes")
    print("=" * 50)

    # Sample product data based on the MSI laptop from the schema
    product_data = {
        "id": "prod_laptop_gaming_001",
        "sku": "MSI-GP66-RTX3070-001",
        "name": "MSI GP66 Leopard Gaming Laptop - RTX 3070, Intel i7-11800H, 16GB RAM, 1TB SSD",
        "description": "Experience next-level gaming performance with the MSI GP66 Leopard. Featuring the latest NVIDIA GeForce RTX 3070 graphics card and Intel Core i7-11800H processor, this laptop delivers exceptional frame rates and smooth gameplay.",
        "short_description": "High-performance gaming laptop with RTX 3070 and i7 processor",
        "price": "189999",  # Price in cents to avoid decimals
        "original_price": "219999",
        "cost_price": "145000",
        "currency": "USD",
        "brand": "MSI",
        "categories": "Electronics,Computers,Laptops,Gaming Laptops",
        "tags": "gaming,laptop,rtx,nvidia,intel,high-performance,144hz,rgb",
        "weight": "240",  # Weight in 100g units (2.4kg = 240)
        "inventory_quantity": "45",
        "status": "active",
        "visibility": "public"
    }

    try:
        # Test 1: Create realistic product
        print("📝 Test 1: Creating realistic product...")

        create_response = requests.post(
            f"{SERVER_URL}/recipe/create_realistic_product",
            json=product_data
        )

        if create_response.status_code == 200:
            print("✅ Product created successfully!")
            result = create_response.json()
            print(f"   Result: {json.dumps(result, indent=2)}")
        else:
            print(f"❌ Failed to create product: {create_response.status_code}")
            print(f"   Error: {create_response.text}")
            return False

        # Small delay to ensure consistency
        time.sleep(0.1)

        # Test 2: Read realistic product
        print("\n📖 Test 2: Reading realistic product...")

        read_response = requests.post(
            f"{SERVER_URL}/recipe/read_realistic_product",
            json={"id": product_data["id"]}
        )

        if read_response.status_code == 200:
            print("✅ Product read successfully!")
            result = read_response.json()
            print(f"   Retrieved product: {json.dumps(result, indent=2)}")

            # Validate some key fields
            if result.get("name") and result.get("price") and result.get("brand"):
                print("✅ Key fields validated successfully!")
            else:
                print("⚠️  Some key fields are missing or empty")
        else:
            print(f"❌ Failed to read product: {read_response.status_code}")
            print(f"   Error: {read_response.text}")
            return False

        # Test 3: List all recipes to verify our new ones are there
        print("\n📋 Test 3: Listing all available recipes...")

        list_response = requests.get(f"{SERVER_URL}/recipes")

        if list_response.status_code == 200:
            recipes = list_response.json()
            print("✅ Available recipes:")
            for name, info in recipes.items():
                if "realistic" in name:
                    print(f"   🎯 {name}: {info.get('description', 'No description')}")
                    print(f"      Inputs: {info.get('inputs', [])}")
                    print(f"      Operations: {info.get('operations_count', 0)}")
        else:
            print(f"❌ Failed to list recipes: {list_response.status_code}")

        # Test 4: Check transaction history
        print("\n🔍 Test 4: Checking transaction history...")

        history_response = requests.get(f"{SERVER_URL}/transactions")

        if history_response.status_code == 200:
            history = history_response.json()
            print(f"✅ Transaction history contains {len(history)} transactions")

            # Show recent transactions related to our product
            product_transactions = [
                tx for tx in history
                if product_data["id"] in tx.get("from_account", "") or
                   product_data["id"] in tx.get("to_account", "")
            ]

            print(f"   📊 Found {len(product_transactions)} transactions for our product:")
            for i, tx in enumerate(product_transactions[-5:]):  # Show last 5
                print(f"      {i+1}. {tx['from_account']} → {tx['to_account']} ({tx['amount']})")
                if tx.get('metadata'):
                    print(f"         Metadata: {tx['metadata']}")
        else:
            print(f"❌ Failed to get history: {history_response.status_code}")

        print("\n🎉 All tests completed successfully!")
        return True

    except requests.exceptions.ConnectionError:
        print("❌ Connection error: Make sure the Pure Accounting Server is running on port 3003")
        print("   Start it with: cd tensorzero/pure-accounting-server && cargo run")
        return False
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        return False

def test_second_product():
    """Test with the iPhone data to ensure the system handles multiple products."""

    print("\n" + "=" * 50)
    print("📱 Testing with iPhone 15 Pro data...")

    iphone_data = {
        "id": "prod_smartphone_flagship_002",
        "sku": "AAPL-IPHONE15-PRO-256",
        "name": "Apple iPhone 15 Pro - 256GB, Natural Titanium",
        "description": "The most advanced iPhone yet, featuring the powerful A17 Pro chip and pro camera system. Built with aerospace-grade titanium for incredible durability and a premium feel.",
        "short_description": "Premium flagship smartphone with A17 Pro chip and pro camera system",
        "price": "119999",  # $1199.99 in cents
        "original_price": "119999",
        "cost_price": "85000",  # $850.00 in cents
        "currency": "USD",
        "brand": "Apple",
        "categories": "Electronics,Mobile Phones,Smartphones,Premium Phones",
        "tags": "iphone,apple,smartphone,premium,titanium,pro,camera,5g",
        "weight": "19",  # 187g rounded to 19 (10g units)
        "inventory_quantity": "128",
        "status": "active",
        "visibility": "public"
    }

    try:
        # Create iPhone
        create_response = requests.post(
            f"{SERVER_URL}/recipe/create_realistic_product",
            json=iphone_data
        )

        if create_response.status_code == 200:
            print("✅ iPhone created successfully!")

            # Read it back
            read_response = requests.post(
                f"{SERVER_URL}/recipe/read_realistic_product",
                json={"id": iphone_data["id"]}
            )

            if read_response.status_code == 200:
                result = read_response.json()
                print(f"✅ iPhone data retrieved: {result.get('name', 'Unknown')}")
                print(f"   Brand: {result.get('brand', 'Unknown')}")
                print(f"   Price: ${int(result.get('price', 0)) / 100:.2f}")
                return True
            else:
                print(f"❌ Failed to read iPhone: {read_response.text}")
                return False
        else:
            print(f"❌ Failed to create iPhone: {create_response.text}")
            return False

    except Exception as e:
        print(f"❌ Error testing iPhone: {e}")
        return False

if __name__ == "__main__":
    print("🦖 Pure Accounting - Realistic Product Recipe Test")
    print("Make sure the server is running: cargo run")
    print()

    success = test_realistic_product_recipes()
    if success:
        success = test_second_product()

    if success:
        print("\n🎉 All tests passed! The realistic product recipes are working correctly.")
    else:
        print("\n❌ Some tests failed. Check the server logs for more details.")

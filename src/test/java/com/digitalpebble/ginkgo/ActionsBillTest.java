// SPDX-License-Identifier: Apache-2.0

package com.digitalpebble.ginkgo;

import com.digitalpebble.ginkgo.model.ActionsBill;
import com.digitalpebble.ginkgo.model.UsageItem;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

import static org.junit.jupiter.api.Assertions.*;

class ActionsBillTest {

    private String loadResource(String name) throws IOException {
        try (InputStream is = getClass().getClassLoader().getResourceAsStream(name)) {
            assertNotNull(is, "Resource not found: " + name);
            return new String(is.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    @Test
    void deserialiseFromJson() throws IOException {
        String json = loadResource("actions_bill.json");
        ActionsBill bill = ActionsBill.fromJson(json);

        assertNotNull(bill);
        List<UsageItem> items = bill.getUsageItems();
        assertEquals(6, items.size());

        UsageItem first = items.get(0);
        assertEquals("2026-01-01T00:00:00Z", first.getDate());
        assertEquals("actions", first.getProduct());
        assertEquals("Actions storage", first.getSku());
        assertEquals(1.978155783, first.getQuantity(), 1e-9);
        assertEquals("GigabyteHours", first.getUnitType());
        assertEquals(0.00033602, first.getPricePerUnit(), 1e-9);
        assertEquals(0.000664386, first.getGrossAmount(), 1e-9);
        assertEquals(0.000664386, first.getDiscountAmount(), 1e-9);
        assertEquals(0.0, first.getNetAmount(), 1e-9);
        assertEquals("DigitalPebble", first.getOrganizationName());
        assertEquals("spruce", first.getRepositoryName());
    }

    @Test
    void serialiseToJson() throws IOException {
        String json = loadResource("actions_bill.json");
        ActionsBill bill = ActionsBill.fromJson(json);

        String serialised = bill.toJson();
        ActionsBill roundTripped = ActionsBill.fromJson(serialised);

        assertEquals(bill.getUsageItems().size(), roundTripped.getUsageItems().size());

        for (int i = 0; i < bill.getUsageItems().size(); i++) {
            UsageItem original = bill.getUsageItems().get(i);
            UsageItem restored = roundTripped.getUsageItems().get(i);
            assertEquals(original.getDate(), restored.getDate());
            assertEquals(original.getSku(), restored.getSku());
            assertEquals(original.getQuantity(), restored.getQuantity(), 1e-9);
            assertEquals(original.getUnitType(), restored.getUnitType());
            assertEquals(original.getRepositoryName(), restored.getRepositoryName());
        }
    }

    @Test
    void readAndWriteFile(@TempDir Path tempDir) throws IOException {
        String json = loadResource("actions_bill.json");
        ActionsBill bill = ActionsBill.fromJson(json);

        Path file = tempDir.resolve("output.json");
        bill.toFile(file);

        assertTrue(Files.exists(file));

        ActionsBill fromFile = ActionsBill.fromFile(file);
        assertEquals(bill.getUsageItems().size(), fromFile.getUsageItems().size());
        assertEquals(
                bill.getUsageItems().get(0).getSku(),
                fromFile.getUsageItems().get(0).getSku()
        );
    }

    @Test
    void emptyUsageItems() {
        ActionsBill bill = ActionsBill.fromJson("{\"usageItems\":[]}");
        assertNotNull(bill.getUsageItems());
        assertTrue(bill.getUsageItems().isEmpty());
    }

    @Test
    void missingUsageItemsDefaultsToEmpty() {
        ActionsBill bill = ActionsBill.fromJson("{}");
        assertNotNull(bill.getUsageItems());
        assertTrue(bill.getUsageItems().isEmpty());
    }
}

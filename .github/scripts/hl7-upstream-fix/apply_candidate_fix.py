#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def apply_inherited_slicing_fix(source: str) -> str:
    old = '''          } else if (dn.current().hasSliceName()) {
            // this is an error unless we're dealing with extensions, which are auto-sliced (for legacy reasons)
            if (diff && "extension".equals(dn.current().getName())) {
              StructureDefinition vsd = new StructureDefinition(); // fake wrapper for placeholder element
              vsd.getDifferential().getElement().add(makeExtensionDefinitionElement(path));
              DefinitionNavigator master = new DefinitionNavigator(context, vsd, diff, followTypes, 0, this.globalPath+"."+tail(path), path, names, null);
              nameMap.put(path, master);
              children.add(master);
              master.slices = new ArrayList<DefinitionNavigator>();
              master.slices.add(dn);
            } else {
              throw new DefinitionException(context.formatMessage(I18nConstants.DN_SLICE_NO_DEFINITION, path));
            }
'''
    new = '''          } else if (dn.current().hasSliceName()) {
            // A differential may constrain a slice whose slicing declaration is inherited.
            // In that case the local differential has no master element, but the generated
            // snapshot does. Use a transient placeholder so the named slice can still be
            // grouped without treating inherited snapshot constraints as local differential
            // content.
            ElementDefinition inheritedSlicing = diff ? makeInheritedSlicingDefinitionElement(path) : null;
            if (inheritedSlicing != null) {
              StructureDefinition vsd = new StructureDefinition(); // fake wrapper for placeholder element
              vsd.getDifferential().getElement().add(inheritedSlicing);
              DefinitionNavigator master = new DefinitionNavigator(context, vsd, diff, followTypes, 0, this.globalPath+"."+tail(path), path, names, null);
              nameMap.put(path, master);
              children.add(master);
              master.slices = new ArrayList<DefinitionNavigator>();
              master.slices.add(dn);
            // extensions remain implicitly sliced by url for legacy reasons
            } else if (diff && "extension".equals(dn.current().getName())) {
              StructureDefinition vsd = new StructureDefinition(); // fake wrapper for placeholder element
              vsd.getDifferential().getElement().add(makeExtensionDefinitionElement(path));
              DefinitionNavigator master = new DefinitionNavigator(context, vsd, diff, followTypes, 0, this.globalPath+"."+tail(path), path, names, null);
              nameMap.put(path, master);
              children.add(master);
              master.slices = new ArrayList<DefinitionNavigator>();
              master.slices.add(dn);
            } else {
              throw new DefinitionException(context.formatMessage(I18nConstants.DN_SLICE_NO_DEFINITION, path));
            }
'''
    source = replace_once(source, old, new, "inherited slice block")

    old_helper = '''  private ElementDefinition makeExtensionDefinitionElement(String path) {
    ElementDefinition ed = new ElementDefinition(path);
    ed.setUserData(UserDataNames.DN_TRANSIENT, "true");
    ed.getSlicing().setRules(SlicingRules.OPEN).setOrdered(false).addDiscriminator().setType(DiscriminatorType.VALUE).setPath("url");
    return ed;
  }
'''
    new_helper = '''  private ElementDefinition makeInheritedSlicingDefinitionElement(String path) {
    if (!structure.hasSnapshot()) {
      return null;
    }
    for (ElementDefinition candidate : structure.getSnapshot().getElement()) {
      if (path.equals(candidate.getPath()) && !candidate.hasSliceName() && candidate.hasSlicing()) {
        ElementDefinition ed = new ElementDefinition(path);
        ed.setUserData(UserDataNames.DN_TRANSIENT, "true");
        ed.setSlicing(candidate.getSlicing().copy());
        return ed;
      }
    }
    return null;
  }

  private ElementDefinition makeExtensionDefinitionElement(String path) {
    ElementDefinition ed = new ElementDefinition(path);
    ed.setUserData(UserDataNames.DN_TRANSIENT, "true");
    ed.getSlicing().setRules(SlicingRules.OPEN).setOrdered(false).addDiscriminator().setType(DiscriminatorType.VALUE).setPath("url");
    return ed;
  }
'''
    return replace_once(source, old_helper, new_helper, "inherited slicing helper")


def apply_content_reference_fix(source: str) -> str:
    old = '''      String path = list().get(i).getPath();
      if (path.startsWith(prefix)) {
'''
    new = '''      String path = list().get(i).getPath();
      // When following a contentReference, slices of the referenced element itself
      // are siblings of that element, not children of the referencing element.
      // The referenced master is intentionally skipped by workingIndex + 1, so skip
      // its same-path named slices as well before walking the referenced children.
      if (childrenFromReference && path.equals(prefix) && list().get(i).hasSliceName()) {
        continue;
      }
      if (path.startsWith(prefix)) {
'''
    return replace_once(source, old, new, "contentReference loop")


def add_tests(test_source: str, inherited: bool, content_reference: bool) -> str:
    if inherited or content_reference:
        old_import = 'import org.hl7.fhir.r5.model.StructureDefinition;\n'
        new_import = '''import org.hl7.fhir.r5.model.ElementDefinition;
import org.hl7.fhir.r5.model.ElementDefinition.DiscriminatorType;
import org.hl7.fhir.r5.model.ElementDefinition.SlicingRules;
import org.hl7.fhir.r5.model.StructureDefinition;
'''
        test_source = replace_once(test_source, old_import, new_import, "test imports")

    tests = []
    if inherited:
        tests.append(r'''
  @Test
  @DisplayName("Inherited differential slice uses snapshot slicing master")
  void inheritedDifferentialSliceUsesSnapshotSlicingMaster() {
    SimpleWorkerContext ctxt = TestingUtilities.getWorkerContext("4.0");
    StructureDefinition sd = new StructureDefinition();
    sd.setType("Observation");

    sd.getDifferential().getElement().add(new ElementDefinition("Observation"));
    sd.getDifferential().getElement().add(
        new ElementDefinition("Observation.category").setSliceName("us-core"));

    sd.getSnapshot().getElement().add(new ElementDefinition("Observation"));
    ElementDefinition master = new ElementDefinition("Observation.category");
    master.getSlicing()
        .setRules(SlicingRules.OPEN)
        .setOrdered(false)
        .addDiscriminator()
        .setType(DiscriminatorType.VALUE)
        .setPath("coding.system");
    sd.getSnapshot().getElement().add(master);
    sd.getSnapshot().getElement().add(
        new ElementDefinition("Observation.category").setSliceName("us-core"));

    DefinitionNavigator dn = new DefinitionNavigator(ctxt, sd, true, false);
    DefinitionNavigator category = dn.childByName("category");
    Assertions.assertNotNull(category);
    Assertions.assertEquals(1, category.slices().size());
    Assertions.assertEquals("us-core", category.slices().get(0).current().getSliceName());
  }
''')

    if content_reference:
        tests.append(r'''
  @Test
  @DisplayName("ContentReference ignores slices of referenced element")
  void contentReferenceIgnoresSlicesOfReferencedElement() {
    SimpleWorkerContext ctxt = TestingUtilities.getWorkerContext("4.0");
    StructureDefinition sd = new StructureDefinition();
    sd.setType("Composition");

    sd.getSnapshot().getElement().add(new ElementDefinition("Composition"));
    ElementDefinition section = new ElementDefinition("Composition.section");
    section.getSlicing()
        .setRules(SlicingRules.OPEN)
        .setOrdered(false)
        .addDiscriminator()
        .setType(DiscriminatorType.VALUE)
        .setPath("code");
    sd.getSnapshot().getElement().add(section);
    sd.getSnapshot().getElement().add(new ElementDefinition("Composition.section.title"));
    sd.getSnapshot().getElement().add(
        new ElementDefinition("Composition.section.section")
            .setContentReference("#Composition.section"));
    sd.getSnapshot().getElement().add(
        new ElementDefinition("Composition.section").setSliceName("sectionProblems"));

    DefinitionNavigator dn = new DefinitionNavigator(ctxt, sd, false, true);
    DefinitionNavigator firstSection = dn.childByName("section");
    Assertions.assertNotNull(firstSection);
    DefinitionNavigator recursiveSection = firstSection.childByName("section");
    Assertions.assertNotNull(recursiveSection);

    Assertions.assertDoesNotThrow(recursiveSection::children);
    Assertions.assertNotNull(recursiveSection.childByName("title"));
    Assertions.assertNotNull(recursiveSection.childByName("section"));
  }
''')

    marker = '\n}'
    if test_source.count(marker) != 1:
        raise SystemExit("test class closing brace was not unique")
    return test_source.replace(marker, ''.join(tests) + marker, 1)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("repo", type=Path)
    parser.add_argument(
        "--variant",
        required=True,
        choices=("inherited", "content-reference", "combined"),
    )
    args = parser.parse_args()

    source_path = args.repo / "org.hl7.fhir.r5/src/main/java/org/hl7/fhir/r5/utils/DefinitionNavigator.java"
    test_path = args.repo / "org.hl7.fhir.r5/src/test/java/org/hl7/fhir/r5/utils/DefinitionNavigatorTests.java"

    source = source_path.read_text(encoding="utf-8")
    tests = test_path.read_text(encoding="utf-8")
    inherited = args.variant in {"inherited", "combined"}
    content_reference = args.variant in {"content-reference", "combined"}

    if inherited:
        source = apply_inherited_slicing_fix(source)
    if content_reference:
        source = apply_content_reference_fix(source)
    tests = add_tests(tests, inherited, content_reference)

    source_path.write_text(source, encoding="utf-8")
    test_path.write_text(tests, encoding="utf-8")


if __name__ == "__main__":
    main()

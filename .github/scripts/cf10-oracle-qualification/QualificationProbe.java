package dev.commandf.oracle.qualification;

import com.fasterxml.jackson.databind.MapperFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import org.hl7.fhir.r5.comparison.ComparisonSession;
import org.hl7.fhir.r5.comparison.ResourceComparer;
import org.hl7.fhir.r5.comparison.ResourceComparer.ResourceComparison;
import org.hl7.fhir.r5.comparison.StructureDefinitionComparer.ProfileComparison;
import org.hl7.fhir.r5.context.IContextResourceLoader;
import org.hl7.fhir.r5.context.SimpleWorkerContext;
import org.hl7.fhir.r5.model.StructureDefinition;
import org.hl7.fhir.utilities.i18n.RenderingI18nContext;
import org.hl7.fhir.utilities.npm.NpmPackage;
import org.hl7.fhir.validation.ValidatorUtils;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;

/**
 * Isolated diagnostic for the CF-10 pinned-HL7 self/cross comparison matrix.
 *
 * <p>The context construction is intentionally identical to the production adapter at the
 * qualified repository head. Unlike the adapter, this diagnostic preserves the exception stored
 * in {@link ResourceComparer.PlaceHolderComparison}, emits a bounded normalized stack, and exits
 * non-zero. It never reconciles the exception into agreement or uncomparable.</p>
 */
public final class QualificationProbe {
  static final int SCHEMA = 1;
  static final String ORACLE_PROJECT = "hapifhir/org.hl7.fhir.core";
  static final String ORACLE_RELEASE = "6.10.2";
  static final String ORACLE_SOURCE_COMMIT =
      "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b";
  static final String CORE_PACKAGE_NAME = "hl7.fhir.r4.core";
  static final String CORE_PACKAGE_VERSION = "4.0.1";
  static final int MAX_STACK_FRAMES = 32;
  static final int MAX_TEXT_CODE_POINTS = 4096;

  private static final ObjectMapper JSON = new ObjectMapper()
      .enable(SerializationFeature.INDENT_OUTPUT)
      .enable(SerializationFeature.ORDER_MAP_ENTRIES_BY_KEYS)
      .enable(MapperFeature.SORT_PROPERTIES_ALPHABETICALLY);

  private QualificationProbe() {
  }

  public static void main(String[] args) throws Exception {
    ProbeResult result;
    try {
      result = qualify(Arguments.parse(args));
    } catch (Throwable error) {
      result = failure("argument_parse", null, null, null, error);
    }
    System.out.write(JSON.writeValueAsBytes(result));
    System.out.write('\n');
    if (!result.status().equals("completed")) {
      System.exit(2);
    }
  }

  static ProbeResult qualify(Arguments args) {
    ContextAndPackage left;
    ContextAndPackage right;
    try {
      left = loadContext(args.corePackage(), args.leftContextPackages(), args.leftPackage());
      right = loadContext(args.corePackage(), args.rightContextPackages(), args.rightPackage());
    } catch (Throwable error) {
      return failure("context_load", null, null, null, error);
    }

    StructureDefinition leftResource;
    StructureDefinition rightResource;
    try {
      leftResource = fetchStructureDefinition(
          left.context(), args.leftUrl(), args.leftVersion(), "left");
      rightResource = fetchStructureDefinition(
          right.context(), args.rightUrl(), args.rightVersion(), "right");
    } catch (Throwable error) {
      return failure("resource_resolution", null, null, null, error);
    }

    ComparisonSession session = new ComparisonSession(
        new RenderingI18nContext(),
        left.context(),
        right.context(),
        "commandF HL7 oracle qualification",
        null,
        null);
    session.setAnnotate(false);

    try {
      ResourceComparison comparison = session.compare(leftResource, rightResource);
      String comparisonClass = comparison == null ? "null" : comparison.getClass().getName();
      if (comparison instanceof ProfileComparison) {
        return completed(comparisonClass, leftResource, rightResource);
      }
      if (comparison instanceof ResourceComparer.PlaceHolderComparison placeholder
          && placeholder.getE() != null) {
        return failure(
            "comparison", comparisonClass, leftResource, rightResource, placeholder.getE());
      }
      return failure(
          "comparison",
          comparisonClass,
          leftResource,
          rightResource,
          new IllegalStateException(
              "HL7 comparison did not return ProfileComparison: " + comparisonClass));
    } catch (Throwable error) {
      return failure("comparison", null, leftResource, rightResource, error);
    }
  }

  private static ContextAndPackage loadContext(
      Path corePath, List<Path> contextPaths, Path sidePath) throws Exception {
    NpmPackage core = loadPackage(corePath);
    requirePackage(core, CORE_PACKAGE_NAME, CORE_PACKAGE_VERSION, "core");

    IContextResourceLoader coreLoader = ValidatorUtils.loaderForVersion(core.fhirVersion());
    SimpleWorkerContext context = new SimpleWorkerContext.SimpleWorkerContextBuilder()
        .withAllowLoadingDuplicates(true)
        .fromPackage(core, coreLoader, false);
    context.setAllowLoadingDuplicates(false);
    context.setCanRunWithoutTerminology(true);

    NpmPackage side = loadPackage(sidePath);
    context.setAllowLoadingDuplicates(true);
    try {
      for (Path contextPath : contextPaths) {
        NpmPackage dependency = loadPackage(contextPath);
        if (samePackage(core, dependency) || samePackage(side, dependency)) {
          continue;
        }
        IContextResourceLoader dependencyLoader =
            ValidatorUtils.loaderForVersion(dependency.fhirVersion());
        dependencyLoader.getTypes().retainAll(Set.of("StructureDefinition"));
        context.loadFromPackage(dependency, dependencyLoader, false);
      }
    } finally {
      context.setAllowLoadingDuplicates(false);
    }
    if (!samePackage(core, side)) {
      IContextResourceLoader sideLoader = ValidatorUtils.loaderForVersion(side.fhirVersion());
      context.loadFromPackage(side, sideLoader, false);
    }
    return new ContextAndPackage(context, side.name(), side.version());
  }

  private static NpmPackage loadPackage(Path path) throws IOException {
    if (!Files.isRegularFile(path)) {
      throw new IOException("package path is not a regular file: " + path);
    }
    try (InputStream input = Files.newInputStream(path)) {
      return NpmPackage.fromPackage(input, path.getFileName().toString(), false);
    }
  }

  private static boolean samePackage(NpmPackage left, NpmPackage right) {
    return Objects.equals(left.name(), right.name())
        && Objects.equals(left.version(), right.version());
  }

  private static void requirePackage(NpmPackage npm, String name, String version, String role) {
    if (!Objects.equals(npm.name(), name) || !Objects.equals(npm.version(), version)) {
      throw new IllegalArgumentException(
          role + " package must be " + name + "#" + version
              + " but was " + npm.name() + "#" + npm.version());
    }
  }

  private static StructureDefinition fetchStructureDefinition(
      SimpleWorkerContext context,
      String url,
      String version,
      String side) {
    String canonical = version == null || version.isBlank() ? url : url + "|" + version;
    StructureDefinition resource = context.fetchResource(StructureDefinition.class, canonical);
    if (resource == null && version != null && !version.isBlank()) {
      resource = context.fetchResource(StructureDefinition.class, url);
    }
    if (resource == null) {
      throw new IllegalArgumentException(
          "unable to resolve " + side + " StructureDefinition " + canonical);
    }
    return resource;
  }

  private static ProbeResult completed(
      String comparisonClass,
      StructureDefinition leftResource,
      StructureDefinition rightResource) {
    return new ProbeResult(
        SCHEMA,
        pinnedOracle(),
        "completed",
        "comparison",
        comparisonClass,
        resourceIdentity(leftResource),
        resourceIdentity(rightResource),
        null,
        null,
        List.of());
  }

  private static ProbeResult failure(
      String phase,
      String comparisonClass,
      StructureDefinition leftResource,
      StructureDefinition rightResource,
      Throwable error) {
    Throwable root = rootCause(error);
    return new ProbeResult(
        SCHEMA,
        pinnedOracle(),
        "exception",
        phase,
        comparisonClass,
        resourceIdentity(leftResource),
        resourceIdentity(rightResource),
        root.getClass().getName(),
        boundedText(root.getMessage() == null ? root.getClass().getSimpleName() : root.getMessage()),
        boundedStack(root));
  }

  private static ProbeOracle pinnedOracle() {
    return new ProbeOracle(ORACLE_PROJECT, ORACLE_RELEASE, ORACLE_SOURCE_COMMIT);
  }

  private static ProbeResource resourceIdentity(StructureDefinition resource) {
    if (resource == null) {
      return null;
    }
    return new ProbeResource(
        emptyToNull(resource.getUrl()),
        emptyToNull(resource.getVersion()),
        emptyToNull(resource.getId()),
        emptyToNull(resource.getType()));
  }

  private static String emptyToNull(String value) {
    return value == null || value.isBlank() ? null : value;
  }

  private static Throwable rootCause(Throwable error) {
    Set<Throwable> seen = java.util.Collections.newSetFromMap(new IdentityHashMap<>());
    Throwable current = error;
    seen.add(current);
    while (current.getCause() != null && seen.add(current.getCause())) {
      current = current.getCause();
    }
    return current;
  }

  private static List<String> boundedStack(Throwable error) {
    List<String> stack = new ArrayList<>();
    StackTraceElement[] frames = error.getStackTrace();
    int limit = Math.min(frames.length, MAX_STACK_FRAMES);
    for (int index = 0; index < limit; index++) {
      StackTraceElement frame = frames[index];
      String file = frame.getFileName() == null ? "unknown" : frame.getFileName();
      stack.add(boundedText(
          frame.getClassName() + "#" + frame.getMethodName()
              + "(" + file + ":" + frame.getLineNumber() + ")"));
    }
    if (frames.length > MAX_STACK_FRAMES) {
      stack.add("... [stack trace truncated]");
    }
    return List.copyOf(stack);
  }

  private static String boundedText(String value) {
    int codePoints = value.codePointCount(0, value.length());
    if (codePoints <= MAX_TEXT_CODE_POINTS) {
      return value;
    }
    int end = value.offsetByCodePoints(0, MAX_TEXT_CODE_POINTS);
    return value.substring(0, end) + "... [text truncated]";
  }

  record ContextAndPackage(SimpleWorkerContext context, String packageName, String packageVersion) {
  }

  record ProbeOracle(String project, String release, String source_commit) {
  }

  record ProbeResource(String url, String version, String id, String type) {
  }

  record ProbeResult(
      int schema,
      ProbeOracle oracle,
      String status,
      String phase,
      String comparison_class,
      ProbeResource left_resource,
      ProbeResource right_resource,
      String exception_class,
      String exception_message,
      List<String> stack_trace) {
  }

  record Arguments(
      Path corePackage,
      Path leftPackage,
      Path rightPackage,
      List<Path> leftContextPackages,
      List<Path> rightContextPackages,
      String leftUrl,
      String leftVersion,
      String rightUrl,
      String rightVersion) {

    static Arguments parse(String[] args) {
      Map<String, String> values = new LinkedHashMap<>();
      List<Path> leftContextPackages = new ArrayList<>();
      List<Path> rightContextPackages = new ArrayList<>();
      for (int index = 0; index < args.length; index += 2) {
        if (index + 1 >= args.length) {
          throw new IllegalArgumentException("missing value for " + args[index]);
        }
        String key = args[index];
        if (!key.startsWith("--")) {
          throw new IllegalArgumentException("unexpected positional argument: " + key);
        }
        String value = args[index + 1];
        if (key.equals("--left-context-package")) {
          leftContextPackages.add(Path.of(value));
        } else if (key.equals("--right-context-package")) {
          rightContextPackages.add(Path.of(value));
        } else if (values.put(key, value) != null) {
          throw new IllegalArgumentException("duplicate argument: " + key);
        }
      }

      List<String> allowed = List.of(
          "--core-package",
          "--left-package",
          "--right-package",
          "--left-url",
          "--left-version",
          "--right-url",
          "--right-version");
      List<String> unknown = new ArrayList<>();
      for (String key : values.keySet()) {
        if (!allowed.contains(key)) {
          unknown.add(key);
        }
      }
      if (!unknown.isEmpty()) {
        unknown.sort(Comparator.naturalOrder());
        throw new IllegalArgumentException("unknown arguments: " + String.join(", ", unknown));
      }

      return new Arguments(
          Path.of(required(values, "--core-package")),
          Path.of(required(values, "--left-package")),
          Path.of(required(values, "--right-package")),
          List.copyOf(leftContextPackages),
          List.copyOf(rightContextPackages),
          required(values, "--left-url"),
          values.get("--left-version"),
          required(values, "--right-url"),
          values.get("--right-version"));
    }

    private static String required(Map<String, String> values, String key) {
      String value = values.get(key);
      if (value == null || value.isBlank()) {
        throw new IllegalArgumentException("missing required argument " + key);
      }
      return value;
    }
  }
}
